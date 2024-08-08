use crate::errors::Error;
use crate::license::License;
use crate::license_file::LicenseFile;
use crate::machine_file::MachineFile;

pub struct Verifier {
    public_key: String,
}

impl Verifier {
    pub fn new(public_key: String) -> Self {
        Self { public_key }
    }

    pub fn verify_license_file(&self, license_file: &LicenseFile) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn verify_machine_file(&self, machine_file: &MachineFile) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn verify_license(&self, license: &License) -> Result<Vec<u8>, Error> {
        unimplemented!()
    }

    pub fn verify_request(&self, request: &reqwest::Request) -> Result<(), Error> {
        unimplemented!()
    }

    pub fn verify_response(&self, response: &reqwest::Response) -> Result<(), Error> {
        unimplemented!()
    }
}
