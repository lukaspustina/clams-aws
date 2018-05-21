extern crate clams;
#[macro_use]
extern crate error_chain;
extern crate futures;
#[macro_use]
extern crate log;
extern crate rusoto_core;
extern crate rusoto_s3;
extern crate rusoto_sts;
extern crate tokio_core;

pub use rusoto_core::Region;

pub mod auth {
    use rusoto_core::{ChainProvider, ProfileProvider, Region};
    use std::time::Duration;
    use tokio_core::reactor::Handle;

    #[derive(Debug, PartialEq)]
    pub struct Profile {
        pub name: Option<String>,
        pub region: Region,
    }

    pub fn credentials_provider(handle: &Handle, profile: &Profile) -> Result<ChainProvider> {
        let mut profile_provider = ProfileProvider::new().chain_err(|| ErrorKind::ProfileProviderFailed)?;
        if let Some(ref name) = profile.name {
            profile_provider.set_profile(name.clone());
        }

        let mut provider = ChainProvider::with_profile_provider(handle, profile_provider);
        provider.set_timeout(Duration::from_secs(60));

        Ok(provider)
    }

    error_chain! {
        errors {
            ProfileProviderFailed {
                description("Failed to create Profile Provider")
            }
        }
    }
}

pub mod s3 {
    use auth::{Profile, credentials_provider};
    use futures::future::Future;
    use futures::stream::Stream;
    use rusoto_core::{ChainProvider, HttpClient};
    use rusoto_s3::{self as s3, GetObjectRequest, PutObjectRequest, S3, StreamingBody};
    use std::default::Default;
    use tokio_core::reactor::Handle;

    pub struct Client<'a> {
        client: s3::S3Client<ChainProvider, HttpClient>,
        handle: &'a Handle,
    }

    impl<'a> Client<'a> {
        pub fn new(handle: &'a Handle, profile: &Profile) -> Result<Self> {
            let client = Client::client(handle, profile)?;

            Ok(Client { client, handle })
        }

        fn client(handle: &Handle, profile: &Profile) -> Result<s3::S3Client<ChainProvider, HttpClient>> {
            let credentials_provider = credentials_provider(handle, profile)?;
            let http_client = HttpClient::new(&handle)
                .chain_err(|| ErrorKind::ClientFailed)?;
            let client = s3::S3Client::new(http_client, credentials_provider, profile.region.clone());

            Ok(client)
        }

        pub fn put_object(&self, bucket: String, key: String, data: Vec<u8>) -> Box<Future<Item=(), Error=Error>> {
            let put = PutObjectRequest {
                    body: Some(data),
                    bucket: bucket.clone(),
                    key: key.clone(),
                    .. Default::default()
                };

            let f = self.client.put_object(&put)
                .map(|_| ())
                .map_err(move |e| Error::with_chain(e, ErrorKind::PutFailed(bucket, key)));

            Box::new(f)
        }

        // TODO: The result type stinks. It should be the body stream or the get_object error
        pub fn get_object(&self, bucket: String, key: String) -> Box<Future<Item=Option<StreamingBody>, Error=Error>> {
            let get = GetObjectRequest {
                    bucket: bucket.clone(),
                    key: key.clone(),
                    .. Default::default()
                };

            let f = self.client.get_object(&get)
                .map(|res| res.body)
                .map_err(move |e| Error::with_chain(e, ErrorKind::GetFailed(bucket.clone(), key.clone())));

            Box::new(f)
        }
    }

    error_chain! {
        errors {
            ClientFailed {
                description("Failed to create S3 client")
            }
            PutFailed(bucket: String, key: String) {
                description("Failed to put object")
                display("Failed to put object to {}/{}", bucket, key)
            }
            GetFailed(bucket: String, key: String) {
                description("Failed to get object")
                display("Failed to get object to {}/{}", bucket, key)
            }
        }

        links {
            Auth(::auth::Error, ::auth::ErrorKind);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
