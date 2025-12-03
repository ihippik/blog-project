// should be run before cargo build
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=proto/blog.proto");
    
    tonic_build::configure()
        .build_server(false)
        .build_client(true)
        .compile(
            &["proto/blog.proto"],
            &["proto"],
        )?;
    Ok(())
}