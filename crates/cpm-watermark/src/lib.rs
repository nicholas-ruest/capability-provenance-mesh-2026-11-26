//! Production adapter for ruvnet/ai-text-watermark.
use cpm_upstream_runtime::{ProbeKind,UpstreamCommand,UpstreamError,UpstreamReceipt};use std::path::PathBuf;
pub const REPOSITORY:&str="ruvnet/ai-text-watermark";pub const REVISION:&str="5871cf5fef6547c97626ba72d976f167562dab2a";
#[derive(Debug,Clone)]pub struct Adapter{command:UpstreamCommand}
impl Adapter{#[must_use]pub fn new(success:Option<PathBuf>,failure:Option<PathBuf>)->Self{Self{command:UpstreamCommand::new(REPOSITORY,REVISION,success,failure)}}pub fn execute(&self,kind:ProbeKind)->Result<UpstreamReceipt,UpstreamError>{self.command.execute(kind)}}
#[cfg(test)]mod tests{use super::*;#[test]fn pins_exact_revision(){assert_eq!(REVISION.len(),40);assert!(Adapter::new(None,None).execute(ProbeKind::Success).is_err())}}
