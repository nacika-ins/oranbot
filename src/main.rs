extern crate mammut;
fn main() {
   try().unwrap();
}

fn try() -> mammut::Result<()> {
  use mammut::Registration;
  use mammut::apps::{AppBuilder, Scope};
  let app = AppBuilder {

      client_name: "oranbot",
      redirect_uris: "urn:ietf:wg:oauth:2.0:oob",
      scopes: Scope::All,
      website: None,
  };
  let mut registration = Registration::new("https://oransns.com")?;

  registration.register(app)?;
  let url = registration.authorise()?;
  println!("承認が必要なURLです: {}", url);
  // Here you now need to open the url in the browser
  // And handle a the redirect url coming back with the code.
  let code = String::from("RETURNED_FROM_BROWSER");

  try {
    let mastodon = registration.create_access_token(code)?;
    println!("{:?}", mastodon.get_home_timeline()?);
  }
  catch e => {
    panic!("アクセストークンが正しくありません");
  }

  Ok(())
}
