//use thiserror::Error;
//
//use crate::{
//    Engine, error,
//    node::{NodeId, NodeRecord},
//};

//#[derive(Error, Debug, Clone)]
//pub enum DocError {
//    #[error("Error parsing document format")]
//    Parse,
//}
//
//const DOC_SEMVER: () = ();
//
//struct DocumentMeta {
//    // Document Semver version
//    version: (),
//    // ISO-8601 date string
//    date: (),
//    // Where the engine left off nonce'ing nodes
//    max_id: NodeId,
//    // User defined meta data based on the client
//    user: (),
//}
//
///// Serialized Grafiek document
//struct Document {
//    nodes: Vec<NodeRecord>,
//    edges: Vec<()>,
//    meta: (),
//}
//
//impl Document {
//    pub fn read<R: std::io::Read>(reader: R) -> Result<(), DocError> {
//        todo!();
//    }
//    pub fn write<W: std::io::Write>(writer: W) -> Result<(), DocError> {
//        todo!();
//    }
//}
//
//impl Engine {
//    pub fn load_document(&mut self, doc: Document) -> Result<(), error::Error> {
//        todo!();
//    }
//
//    pub fn save_document(&mut self) -> Result<Document, error::Error> {
//        todo!();
//    }
//}
