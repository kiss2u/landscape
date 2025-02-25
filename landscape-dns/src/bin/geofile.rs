use landscape_dns::protos::geo::GeoIPListOwned;

#[tokio::main]
async fn main() {
    let home_path = homedir::my_home().unwrap().unwrap().join(".landscape-router");
    let geo_file_path = home_path.join("geoip.dat");

    let data = tokio::fs::read(geo_file_path).await.unwrap();
    let list = GeoIPListOwned::try_from(data).unwrap();

    let mut sum = 0;
    for entry in list.entry.iter() {
        println!("{:?}", entry.country_code);
        if entry.country_code == "cn".to_uppercase() {
            println!("{:?}", entry.cidr.len());
        } else {
            sum += entry.cidr.len()
        }
        println!("reverse_match : {:?}", entry.reverse_match);
        // if entry.reverse_match {
        //     println!("reverse_match : {:?}", entry.cidr);
        // }
    }
    println!("other count: {sum:?}");
}
