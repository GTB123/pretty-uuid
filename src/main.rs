use tonic::transport::Server;

pub mod server;
pub mod store;
pub mod prefix_mappings;

mod store_proto {
    include!("store.rs");

    pub(crate) const FILE_DESCRIPTOR_SET: &[u8] =
        tonic::include_file_descriptor_set!("store_descriptor");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "127.0.0.1:9001".parse()?;

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(store_proto::FILE_DESCRIPTOR_SET)
        .build()
        .unwrap();
    let uuid_service = server::UuidService::default();
    Server::builder()
        .add_service(store::uuid_server::UuidServer::new(uuid_service))
        .add_service(reflection_service)
        .serve(addr)
        .await?;
    Ok(())
}
