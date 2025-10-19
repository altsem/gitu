#[derive(Debug, PartialEq, Eq)]
pub(crate) struct Status {
    pub branch_status: BranchStatus,
    pub files: Vec<StatusFile>,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct BranchStatus {
    pub local: Option<String>,
    pub remote: Option<String>,
    pub ahead: u32,
    pub behind: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct StatusFile {
    pub status_code: [char; 2],
    pub path: String,
    pub new_path: Option<String>,
}

impl StatusFile {
    pub fn is_untracked(&self) -> bool {
        self.status_code == ['?', '?']
    }
}
