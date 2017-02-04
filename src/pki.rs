use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};

use hyper::Client;
use rusoto::ChainProvider;
use serde_json::from_slice;
use tempdir::TempDir;

use encryption::Encryptor;
use error::{KawsError, KawsResult};

pub struct Certificate(Vec<u8>);

pub struct CertificateAuthority {
    cert: Certificate,
    key: PrivateKey,
}

pub struct CertificateSigningRequest(Vec<u8>);

pub struct PrivateKey(Vec<u8>);

#[derive(Deserialize)]
struct CfsslGencertResponse {
    cert: Vec<u8>,
    key: Vec<u8>,
}

#[derive(Deserialize)]
struct CfsslSignResponse {
    cert: Vec<u8>,
}

#[derive(Deserialize)]
struct CfsslGenkeyResponse {
    csr: Vec<u8>,
    key: Vec<u8>,
}

impl Certificate {
    pub fn write_to_file(&self, file_path: &str) -> KawsResult {
        let mut file = File::create(file_path)?;
        file.write_all(self.as_bytes())?;

        Ok(None)
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for Certificate {
    fn from(vec: Vec<u8>) -> Self {
        Certificate(vec)
    }
}

impl CertificateAuthority {
    pub fn generate(common_name: &str) -> Result<Self, KawsError> {
        let mut command = Command::new("cfssl");

        command.args(&[
            "gencert",
            "-initca",
            "-",
        ]);

        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command.spawn()?;

        match child.stdin.as_mut() {
            Some(stdin) => {
                stdin.write_all(
                    format!(
                        r#"{{"CN":"{}","key":{{"algo":"rsa","size":2048}}}}}}"#,
                        common_name
                    ).as_bytes()
                )?;
            }
            None => {
                return Err(
                    KawsError::new("failed to acquire handle to stdin of child process".to_owned())
                );
            }
        }

        let output = child.wait_with_output()?;

        if output.status.success() {
            let raw: CfsslGencertResponse = from_slice(&output.stdout)?;

            Ok(raw.into())
        } else {
            Err(KawsError::new("Execution of `cfssl genkey` failed.".to_owned()))
        }
    }

    pub fn generate_cert(&self, common_name: &str, san: Option<&[&str]>)
    -> Result<(Certificate, PrivateKey), KawsError> {
        let (tempdir, cert_path, key_path) = self.temporary_write()?;

        let mut command = Command::new("cfssl");

        command.args(&[
            "gencert",
            "-ca",
            &cert_path,
            "-ca-key",
            &key_path,
            "-",
        ]);

        if let Some(san) = san {
            command.args(&[
                "-hostname",
                &san.join(","),
            ]);
        }

        command.arg("-");

        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command.spawn()?;

        match child.stdin.as_mut() {
            Some(stdin) => {
                stdin.write_all(
                    format!(
                        r#"{{"CN":"{}","key":{{"algo":"rsa","size":2048}}}}}}"#,
                        common_name
                    ).as_bytes()
                )?;
            }
            None => {
                return Err(
                    KawsError::new("failed to acquire handle to stdin of child process".to_owned())
                );
            }
        }

        let output = child.wait_with_output()?;

        let result = if output.status.success() {
            let raw: CfsslGencertResponse = from_slice(&output.stdout)?;

            Ok((raw.cert.into(), raw.key.into()))
        } else {
            Err(KawsError::new("Execution of `cfssl gencert` failed.".to_owned()))
        };

        tempdir.close()?;

        result
    }

    pub fn sign(&self, csr: &CertificateSigningRequest) -> Result<Certificate, KawsError> {
        let (tempdir, cert_path, key_path) = self.temporary_write()?;

        let mut command = Command::new("cfssl");

        command.args(&[
            "sign",
            "-ca",
            &cert_path,
            "-ca-key",
            &key_path,
            "-"
        ]);

        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command.spawn()?;

        match child.stdin.as_mut() {
            Some(stdin) => {
                stdin.write_all(csr.as_bytes())?;
            }
            None => {
                return Err(
                    KawsError::new("failed to acquire handle to stdin of child process".to_owned())
                );
            }
        }

        let output = child.wait_with_output()?;

        let result = if output.status.success() {
            let response: CfsslSignResponse = from_slice(&output.stdout)?;

            Ok(response.cert.into())
        } else {
            Err(KawsError::new("Execution of `cfssl gencert` failed.".to_owned()))
        };

        tempdir.close()?;

        result
    }

    pub fn write_to_files(
        &self,
        encryptor: &mut Encryptor<ChainProvider, Client>,
        cert_file_path: &str,
        key_file_path: &str,
    ) -> KawsResult {
        let mut cert_file = File::create(cert_file_path)?;
        cert_file.write_all(self.as_bytes())?;

        encryptor.encrypt_and_write_file(self.key.as_bytes(), key_file_path)?;

        Ok(None)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.cert.as_bytes()
    }

    // Private

    fn temporary_write(&self) -> Result<(TempDir, String, String), KawsError> {
        let tempdir = TempDir::new("kaws")?;

        let cert_path = tempdir.path().join("cert.pem");
        let key_path = tempdir.path().join("key.pem");
        let cert_path_string = match cert_path.to_str() {
            Some(value) => value.to_owned(),
            None => return Err(KawsError::new("Temporary path was invalid UTF-8".to_owned())),
        };
        let key_path_string = match key_path.to_str() {
            Some(value) => value.to_owned(),
            None => return Err(KawsError::new("Temporary path was invalid UTF-8".to_owned())),
        };
        let mut cert_file = File::create(cert_path)?;
        let mut key_file = File::create(key_path)?;
        cert_file.write_all(self.cert.as_bytes())?;
        key_file.write_all(self.key.as_bytes())?;

        Ok((tempdir, cert_path_string, key_path_string))
    }

}

impl From<CfsslGencertResponse> for CertificateAuthority {
    fn from(raw: CfsslGencertResponse) -> Self {
        CertificateAuthority {
            cert: raw.cert.into(),
            key: raw.key.into(),
        }
    }
}

impl CertificateSigningRequest {
    pub fn generate(common_name: &str)
    -> Result<(CertificateSigningRequest, PrivateKey), KawsError> {
        let mut command = Command::new("cfssl");

        command.args(&[
            "genkey",
            "-",
        ]);

        command.stdin(Stdio::piped());
        command.stdout(Stdio::piped());
        command.stderr(Stdio::piped());

        let mut child = command.spawn()?;

        match child.stdin.as_mut() {
            Some(stdin) => {
                stdin.write_all(
                    format!(
                        r#"{{"CN":"{}","key":{{"algo":"rsa","size":2048}}}}}}"#,
                        common_name
                    ).as_bytes(),
                )?;
            }
            None => {
                return Err(
                    KawsError::new("failed to acquire handle to stdin of child process".to_owned())
                );
            }
        };

        let output = child.wait_with_output()?;

        if output.status.success() {
            let raw: CfsslGenkeyResponse = from_slice(&output.stdout)?;

            Ok((CertificateSigningRequest(raw.csr), PrivateKey(raw.key)))
        } else {
            Err(KawsError::new("Execution of `cfssl genkey` failed.".to_owned()))
        }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for CertificateSigningRequest {
    fn from(vec: Vec<u8>) -> Self {
        CertificateSigningRequest(vec)
    }
}

impl PrivateKey {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn write_to_file(
        &self,
        encryptor: &mut Encryptor<ChainProvider, Client>,
        file_path: &str,
    ) -> KawsResult {
        encryptor.encrypt_and_write_file(self.as_bytes(), file_path)?;

        Ok(None)
    }
}

impl From<Vec<u8>> for PrivateKey {
    fn from(vec: Vec<u8>) -> Self {
        PrivateKey(vec)
    }
}
