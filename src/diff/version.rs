// Tue Jan 15 2026 - Alex

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

/// Semantic version representation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub prerelease: Option<String>,
    pub build: Option<String>,
}

impl Version {
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            prerelease: None,
            build: None,
        }
    }

    pub fn with_prerelease(mut self, pre: &str) -> Self {
        self.prerelease = Some(pre.to_string());
        self
    }

    pub fn with_build(mut self, build: &str) -> Self {
        self.build = Some(build.to_string());
        self
    }

    /// Parse version from string
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().trim_start_matches('v').trim_start_matches('V');
        
        // Handle build metadata
        let (version_part, build) = if let Some(idx) = s.find('+') {
            (&s[..idx], Some(s[idx+1..].to_string()))
        } else {
            (s, None)
        };

        // Handle prerelease
        let (version_part, prerelease) = if let Some(idx) = version_part.find('-') {
            (&version_part[..idx], Some(version_part[idx+1..].to_string()))
        } else {
            (version_part, None)
        };

        // Parse major.minor.patch
        let parts: Vec<&str> = version_part.split('.').collect();
        
        let major = parts.first()?.parse().ok()?;
        let minor = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
        let patch = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

        Some(Self {
            major,
            minor,
            patch,
            prerelease,
            build,
        })
    }

    /// Check if this is a major version bump from other
    pub fn is_major_bump_from(&self, other: &Version) -> bool {
        self.major > other.major
    }

    /// Check if this is a minor version bump from other
    pub fn is_minor_bump_from(&self, other: &Version) -> bool {
        self.major == other.major && self.minor > other.minor
    }

    /// Check if this is a patch version bump from other
    pub fn is_patch_bump_from(&self, other: &Version) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch > other.patch
    }

    /// Get the bump type from another version
    pub fn bump_type_from(&self, other: &Version) -> BumpType {
        if self.major > other.major {
            BumpType::Major
        } else if self.minor > other.minor {
            BumpType::Minor
        } else if self.patch > other.patch {
            BumpType::Patch
        } else if self == other {
            BumpType::None
        } else {
            BumpType::Downgrade
        }
    }

    pub fn is_prerelease(&self) -> bool {
        self.prerelease.is_some()
    }
}

impl Default for Version {
    fn default() -> Self {
        Self::new(0, 0, 0)
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.major.cmp(&other.major) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.minor.cmp(&other.minor) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.patch.cmp(&other.patch) {
            Ordering::Equal => {}
            ord => return ord,
        }
        
        // Prerelease versions have lower precedence
        match (&self.prerelease, &other.prerelease) {
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
            (None, None) => Ordering::Equal,
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)?;
        if let Some(ref pre) = self.prerelease {
            write!(f, "-{}", pre)?;
        }
        if let Some(ref build) = self.build {
            write!(f, "+{}", build)?;
        }
        Ok(())
    }
}

impl FromStr for Version {
    type Err = VersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Version::parse(s).ok_or(VersionParseError)
    }
}

/// Version parse error
#[derive(Debug)]
pub struct VersionParseError;

impl fmt::Display for VersionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse version")
    }
}

impl std::error::Error for VersionParseError {}

/// Version bump type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpType {
    Major,
    Minor,
    Patch,
    None,
    Downgrade,
}

impl BumpType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BumpType::Major => "major",
            BumpType::Minor => "minor",
            BumpType::Patch => "patch",
            BumpType::None => "none",
            BumpType::Downgrade => "downgrade",
        }
    }

    pub fn is_breaking(&self) -> bool {
        matches!(self, BumpType::Major)
    }
}

/// Extended version info
#[derive(Debug, Clone)]
pub struct VersionInfo {
    pub version: Version,
    pub name: Option<String>,
    pub release_date: Option<String>,
    pub hash: Option<String>,
    pub size: Option<usize>,
    pub notes: Option<String>,
}

impl VersionInfo {
    pub fn new(version: Version) -> Self {
        Self {
            version,
            name: None,
            release_date: None,
            hash: None,
            size: None,
            notes: None,
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn with_date(mut self, date: &str) -> Self {
        self.release_date = Some(date.to_string());
        self
    }

    pub fn with_hash(mut self, hash: &str) -> Self {
        self.hash = Some(hash.to_string());
        self
    }

    pub fn with_size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }
}

impl fmt::Display for VersionInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version)?;
        if let Some(ref name) = self.name {
            write!(f, " ({})", name)?;
        }
        if let Some(ref date) = self.release_date {
            write!(f, " - {}", date)?;
        }
        Ok(())
    }
}

/// Comparison between two versions
#[derive(Debug, Clone)]
pub struct VersionComparison {
    pub old: VersionInfo,
    pub new: VersionInfo,
    pub bump_type: BumpType,
    pub days_between: Option<i64>,
    pub size_delta: Option<i64>,
}

impl VersionComparison {
    pub fn new(old: VersionInfo, new: VersionInfo) -> Self {
        let bump_type = new.version.bump_type_from(&old.version);
        let size_delta = match (old.size, new.size) {
            (Some(o), Some(n)) => Some(n as i64 - o as i64),
            _ => None,
        };

        Self {
            old,
            new,
            bump_type,
            days_between: None,
            size_delta,
        }
    }

    pub fn is_upgrade(&self) -> bool {
        self.new.version > self.old.version
    }

    pub fn is_downgrade(&self) -> bool {
        self.new.version < self.old.version
    }

    pub fn version_delta(&self) -> String {
        format!("{} -> {}", self.old.version, self.new.version)
    }
}

impl fmt::Display for VersionComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Version Comparison")?;
        writeln!(f, "  Old: {}", self.old)?;
        writeln!(f, "  New: {}", self.new)?;
        writeln!(f, "  Bump: {:?}", self.bump_type)?;
        if let Some(delta) = self.size_delta {
            writeln!(f, "  Size delta: {:+} bytes", delta)?;
        }
        Ok(())
    }
}

/// Version range for compatibility checking
#[derive(Debug, Clone)]
pub struct VersionRange {
    pub min: Option<Version>,
    pub max: Option<Version>,
    pub min_inclusive: bool,
    pub max_inclusive: bool,
}

impl VersionRange {
    pub fn any() -> Self {
        Self {
            min: None,
            max: None,
            min_inclusive: true,
            max_inclusive: true,
        }
    }

    pub fn exact(version: Version) -> Self {
        Self {
            min: Some(version.clone()),
            max: Some(version),
            min_inclusive: true,
            max_inclusive: true,
        }
    }

    pub fn at_least(version: Version) -> Self {
        Self {
            min: Some(version),
            max: None,
            min_inclusive: true,
            max_inclusive: true,
        }
    }

    pub fn up_to(version: Version) -> Self {
        Self {
            min: None,
            max: Some(version),
            min_inclusive: true,
            max_inclusive: false,
        }
    }

    pub fn between(min: Version, max: Version) -> Self {
        Self {
            min: Some(min),
            max: Some(max),
            min_inclusive: true,
            max_inclusive: false,
        }
    }

    pub fn contains(&self, version: &Version) -> bool {
        let above_min = match &self.min {
            None => true,
            Some(min) => {
                if self.min_inclusive {
                    version >= min
                } else {
                    version > min
                }
            }
        };

        let below_max = match &self.max {
            None => true,
            Some(max) => {
                if self.max_inclusive {
                    version <= max
                } else {
                    version < max
                }
            }
        };

        above_min && below_max
    }
}

impl fmt::Display for VersionRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (&self.min, &self.max) {
            (None, None) => write!(f, "*"),
            (Some(min), None) => write!(f, ">={}", min),
            (None, Some(max)) => write!(f, "<{}", max),
            (Some(min), Some(max)) if min == max => write!(f, "={}", min),
            (Some(min), Some(max)) => write!(f, ">={}, <{}", min, max),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
        
        let v = Version::parse("v2.0.0-beta+build123").unwrap();
        assert_eq!(v.major, 2);
        assert_eq!(v.prerelease, Some("beta".to_string()));
        assert_eq!(v.build, Some("build123".to_string()));
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(2, 0, 0);
        let v3 = Version::new(1, 1, 0);
        
        assert!(v2 > v1);
        assert!(v3 > v1);
        assert!(v2 > v3);
    }

    #[test]
    fn test_bump_type() {
        let v1 = Version::new(1, 0, 0);
        let v2 = Version::new(2, 0, 0);
        let v3 = Version::new(1, 1, 0);
        let v4 = Version::new(1, 0, 1);
        
        assert_eq!(v2.bump_type_from(&v1), BumpType::Major);
        assert_eq!(v3.bump_type_from(&v1), BumpType::Minor);
        assert_eq!(v4.bump_type_from(&v1), BumpType::Patch);
    }

    #[test]
    fn test_version_range() {
        let range = VersionRange::between(Version::new(1, 0, 0), Version::new(2, 0, 0));
        
        assert!(range.contains(&Version::new(1, 5, 0)));
        assert!(!range.contains(&Version::new(2, 0, 0)));
        assert!(!range.contains(&Version::new(0, 9, 0)));
    }
}
