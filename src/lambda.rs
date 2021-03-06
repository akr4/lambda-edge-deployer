use chrono::{DateTime, Utc};
use regex::Regex;
use rusoto_core::Region;
use rusoto_lambda::{
    DeleteFunctionRequest, Lambda, LambdaClient, ListVersionsByFunctionRequest,
    UpdateFunctionCodeRequest,
};
use std::error::Error;
use std::fs::File;
use std::io::{BufReader, Read};

type Result<T> = std::result::Result<T, failure::Error>;

pub struct UpdateFunctionResult {
    function_name: String,
    pub version: String,
}

impl UpdateFunctionResult {
    pub fn tag_name(&self) -> String {
        format!("{}@{}", self.function_name, self.version)
    }
}

pub fn update_function(function_name: &str, zip_file: File) -> Result<UpdateFunctionResult> {
    let client = LambdaClient::new(Region::UsEast1);
    let mut buf = Vec::new();
    let mut reader = BufReader::new(zip_file);
    reader.read_to_end(&mut buf)?;
    let config = client
        .update_function_code(UpdateFunctionCodeRequest {
            function_name: function_name.to_owned(),
            zip_file: Some(buf.into()),
            publish: Some(true),
            ..Default::default()
        })
        .sync()?;

    if config.function_name.is_none() || config.version.is_none() {
        return Err(failure::err_msg("no function_name or version"));
    } else {
        Ok(UpdateFunctionResult {
            function_name: config.function_name.unwrap(),
            version: config.version.unwrap(),
        })
    }
}

pub struct RemoveOldVersionsResult {
    pub deleted_versions: Vec<String>,
    pub failures: Vec<DeleteFailure>,
}

pub struct DeleteFailure {
    pub version: String,
    pub reason: String,
}

pub fn remove_old_versions(
    function_name: &str,
    current_version: &str,
) -> Result<RemoveOldVersionsResult> {
    let client = LambdaClient::new(Region::UsEast1);

    let response = client
        .list_versions_by_function(ListVersionsByFunctionRequest {
            function_name: function_name.to_owned(),
            ..Default::default()
        })
        .sync()?;

    let mut deleted_versions = Vec::new();
    let mut failures = Vec::new();

    if let Some(versions) = response.versions {
        for config in versions {
            if let (Some(version), Some(last_modified)) = (config.version, config.last_modified) {
                let re = Regex::new(r"^[0-9]+$").unwrap();
                if !re.is_match(&version) {
                    continue;
                }
                if version == current_version {
                    continue;
                }
                let datetime = DateTime::parse_from_str(&last_modified, "%Y-%m-%dT%H:%M:%S.%3f%z")?
                    .with_timezone(&Utc);
                let duration = Utc::now() - datetime;
                if duration.num_days() >= 0 {
                    match client
                        .delete_function(DeleteFunctionRequest {
                            function_name: function_name.to_owned(),
                            qualifier: Some(version.clone()),
                            ..Default::default()
                        })
                        .sync()
                    {
                        Ok(_) => {
                            deleted_versions.push(version);
                        }
                        Err(e) => {
                            failures.push(DeleteFailure {
                                version,
                                reason: e.description().to_owned(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(RemoveOldVersionsResult {
        deleted_versions,
        failures,
    })
}
