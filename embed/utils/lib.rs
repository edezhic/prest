#![forbid(unsafe_code)]

use sha2::Digest;
use std::borrow::Cow;
use std::path::Path;
use std::time::SystemTime;
use std::{fs, io};

/// Utility used for debug and lazy embeddings
#[doc(hidden)]
#[cfg_attr(all(debug_assertions, not(feature = "lazy-embed")), allow(unused))]
pub struct FileEntry {
  pub rel_path: String,
  pub full_canonical_path: String,
}

#[doc(hidden)]
pub fn is_path_included(rel_path: &str, includes: &[&str], excludes: &[&str]) -> bool {
  use globset::Glob;

  // ignore path matched by exclusion pattern
  for exclude in excludes {
    let pattern = Glob::new(exclude)
      .unwrap_or_else(|_| panic!("invalid exclude pattern '{}'", exclude))
      .compile_matcher();

    if pattern.is_match(rel_path) {
      return false;
    }
  }

  // accept path if no includes provided
  if includes.is_empty() {
    return true;
  }

  // accept path if matched by inclusion pattern
  for include in includes {
    let pattern = Glob::new(include)
      .unwrap_or_else(|_| panic!("invalid include pattern '{}'", include))
      .compile_matcher();

    if pattern.is_match(rel_path) {
      return true;
    }
  }

  false
}

/// Utility that finds files to embed
#[doc(hidden)]
#[cfg_attr(all(debug_assertions, not(feature = "lazy-embed")), allow(unused))]
pub fn get_files<'patterns>(folder_path: String, includes: &'patterns [&str], excludes: &'patterns [&str]) -> impl Iterator<Item = FileEntry> + 'patterns {
  walkdir::WalkDir::new(&folder_path)
    .follow_links(true)
    .sort_by_file_name()
    .into_iter()
    // ignore errors
    .filter_map(|e| e.ok())
    // only files
    .filter(|e| e.file_type().is_file())
    // respect includes and excludes
    .filter_map(move |e| {
      let rel_path = path_to_str(e.path().strip_prefix(&folder_path).unwrap());
      let full_canonical_path = path_to_str(std::fs::canonicalize(e.path()).expect("Could not get canonical path"));

      let rel_path = if std::path::MAIN_SEPARATOR == '\\' {
        rel_path.replace('\\', "/")
      } else {
        rel_path
      };

      if is_path_included(&rel_path, includes, excludes) {
        Some(FileEntry { rel_path, full_canonical_path })
      } else {
        None
      }
    })
}

/// A file embedded into the binary
#[derive(Clone)]
pub struct EmbeddedFile {
  pub data: Cow<'static, [u8]>,
  pub metadata: EmbeddedFileMetadata,
}

/// EmbeddedFileMetadata about an embedded file
#[doc(hidden)]
#[derive(Clone)]
pub struct EmbeddedFileMetadata {
  hash: [u8; 32],
  last_modified: Option<u64>,
  mimetype: Cow<'static, str>,
}

impl EmbeddedFileMetadata {
  #[doc(hidden)]
  pub const fn __rust_embed_new(hash: [u8; 32], last_modified: Option<u64>, mimetype: &'static str) -> Self {
    Self {
      hash,
      last_modified,
      mimetype: Cow::Borrowed(mimetype),
    }
  }

  /// The SHA256 hash of the file
  pub fn sha256_hash(&self) -> [u8; 32] {
    self.hash
  }

  /// The last modified date in seconds since the UNIX epoch. If the underlying
  /// platform/file-system does not support this, None is returned.
  pub fn last_modified(&self) -> Option<u64> {
    self.last_modified
  }

  /// The mime type of the file
  pub fn mimetype(&self) -> &str {
    &self.mimetype
  }
}

#[doc(hidden)]
pub fn read_file_from_fs(file_path: &Path) -> io::Result<EmbeddedFile> {
  let data = fs::read(file_path)?;
  let data = Cow::from(data);

  let mut hasher = sha2::Sha256::new();
  hasher.update(&data);
  let hash: [u8; 32] = hasher.finalize().into();

  let source_date_epoch = match std::env::var("SOURCE_DATE_EPOCH") {
    Ok(value) => value.parse::<u64>().map_or(None, |v| Some(v)),
    Err(_) => None,
  };

  let last_modified = fs::metadata(file_path)?.modified().ok().map(|last_modified| {
    last_modified
      .duration_since(SystemTime::UNIX_EPOCH)
      .expect("Time before the UNIX epoch is unsupported")
      .as_secs()
  });

  let mimetype = mime_guess::from_path(file_path).first_or_octet_stream().to_string();

  Ok(EmbeddedFile {
    data,
    metadata: EmbeddedFileMetadata {
      hash,
      last_modified: source_date_epoch.or(last_modified),
      mimetype: mimetype.into(),
    },
  })
}

fn path_to_str<P: AsRef<std::path::Path>>(p: P) -> String {
  p.as_ref().to_str().expect("Path does not have a string representation").to_owned()
}
