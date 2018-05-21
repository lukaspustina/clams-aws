extern crate clams_aws;
extern crate futures;
extern crate spectral;
extern crate tokio_core;

#[cfg(test)]
mod s3 {
    use clams_aws::Region;
    use clams_aws::auth::Profile;
    use clams_aws::s3::Client;
    use futures::prelude::*;
    use tokio_core::reactor::Core;

    use spectral::prelude::*;

    #[test]
    #[ignore]
    fn put_object_okay() {
        let mut core = Core::new().unwrap();

        let profile = Profile {
            name: Some("pustina_de".to_owned()),
            region: Region::EuCentral1,
        };

        let handle = core.handle();
        let client = Client::new(&handle, &profile).expect("Failed to create S3 client");

        let bucket = "de.pustina.sandbox".to_owned();
        let key = "dev/clams_aws/s3/put_object_okay".to_owned();

        let f = client.put_object(bucket, key, "This is just a punk-rock song".as_bytes().to_vec());

        let res = core.run(f);

        assert_that(&res).is_ok();
    }

    #[test]
    #[ignore]
    fn get_object_okay() {
        let mut core = Core::new().unwrap();

        let profile = Profile {
            name: Some("pustina_de".to_owned()),
            region: Region::EuCentral1,
        };

        let handle = core.handle();
        let client = Client::new(&handle, &profile).expect("Failed to create S3 client");

        let bucket = "de.pustina.sandbox".to_owned();
        let key = "dev/clams_aws/s3/put_object_okay".to_owned();

        let f = client.get_object(bucket, key);

        let res = core.run(f);
        assert_that(&res).is_ok().is_some();

        let body = res.unwrap().unwrap() // Safe
            .concat2();
        let body_res = core.run(body).unwrap();
        assert_that(&body_res.len()).is_equal_to(29);
        assert_that(&body_res).is_equal_to("This is just a punk-rock song".as_bytes().to_vec());
    }
}
