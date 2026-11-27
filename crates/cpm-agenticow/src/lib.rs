//! Production adapter for ruvnet/agenticow.
use cpm_upstream_runtime::{ProbeKind,UpstreamCommand,UpstreamError,UpstreamReceipt};use std::path::PathBuf;
pub const REPOSITORY:&str="ruvnet/agenticow";pub const REVISION:&str="a6467863e250371562c2e0c83f5f993459607746";
#[derive(Debug,Clone)]pub struct Adapter{command:UpstreamCommand}
impl Adapter{#[must_use]pub fn new(success:Option<PathBuf>,failure:Option<PathBuf>)->Self{Self{command:UpstreamCommand::new(REPOSITORY,REVISION,success,failure)}}pub fn execute(&self,kind:ProbeKind)->Result<UpstreamReceipt,UpstreamError>{self.command.execute(kind)}}
#[cfg(test)]mod tests{use super::*;#[test]fn pins_exact_revision(){assert_eq!(REVISION.len(),40);assert!(Adapter::new(None,None).execute(ProbeKind::Success).is_err())}}
