use crate::Result;
use tokio_rustls::rustls;

use rustls::{
    ServerConfig,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};

pub fn server_config(certs: String, private_key: String) -> Result<ServerConfig> {
    let cert_chain = CertificateDer::pem_file_iter(certs)
        .unwrap()
        .collect::<Result<Vec<_>, _>>()?;

    let key_der = PrivateKeyDer::from_pem_file(private_key)?;

    Ok(rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, key_der)?)
}
