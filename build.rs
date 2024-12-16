use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Output configuration arguments for ESP-IDF
    embuild::build::CfgArgs::output_propagated("ESP_IDF")?;
    embuild::build::LinkArgs::output_propagated("ESP_IDF")?;

    // Check for OpenSSL configuration environment variables
    if let Ok(openssl_config_dir) = env::var("OPENSSL_CONFIG_DIR") {
        println!("cargo:rustc-link-search=native={}", openssl_config_dir);
        println!("cargo:rustc-link-lib=ssl"); // Link against OpenSSL SSL library
        println!("cargo:rustc-link-lib=crypto"); // Link against OpenSSL Crypto library
    }

    Ok(())
}
