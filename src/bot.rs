use mammut::Mastodon;
use mammut::entities::prelude::*;

pub fn exec(mastodon: &Mastodon) -> () {

    // println!("{:?}", mastodon.get_home_timeline().unwrap());

    for status in mastodon.get_home_timeline().unwrap() {
        let status: Status = status;
        println!("{:?}", status.content);
    }


}