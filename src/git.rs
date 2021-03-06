use std::path::Path;

type Result<T> = std::result::Result<T, failure::Error>;

pub struct Repository {
    repo: git2::Repository,
}

#[derive(Debug)]
pub struct Tag(String);

impl Tag {
    pub fn new(s: &str) -> Self {
        Tag(s.to_owned())
    }

    pub fn name(&self) -> String {
        self.0.clone()
    }
}

impl Repository {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let repo = git2::Repository::open(path)?;
        Ok(Self { repo })
    }

    pub fn has_uncommitted_files(&self) -> Result<bool> {
        let statuses = self.repo.statuses(None)?;
        Ok(statuses
            .iter()
            .any(|x| x.status() == git2::Status::WT_MODIFIED))
    }

    pub fn tag(&self, tag: &str) -> Result<Tag> {
        let head = self.repo.revparse_single("HEAD")?;
        self.repo.tag_lightweight(&tag, &head, false)?;
        Ok(Tag::new(tag))
    }

    pub fn get_tags(&self) -> Result<Vec<Tag>> {
        let head = self.repo.revparse_single("HEAD")?;
        let tags = self
            .repo
            .references()?
            .filter_map(|x| match x {
                Ok(r) => {
                    if r.is_tag() {
                        if let Some(oid) = r.target() {
                            if oid == head.id() {
                                if let Some(name) = r.name() {
                                    return Some(Tag(name["refs/tags/".len()..].to_owned()));
                                }
                            }
                        }
                    }
                    return None;
                }
                Err(_) => None,
            })
            .collect();
        Ok(tags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_tags() {
        let repo = Repository::open("repos/a").unwrap();
        println!("{:?}", repo.get_tags().unwrap());
    }
}
