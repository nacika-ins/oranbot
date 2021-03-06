use regex::Regex;
use mammut::Mastodon;
use mammut::StatusBuilder;
use mammut::entities::notification::NotificationType;
use mammut::entities::prelude::*;
use std::{thread, time};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::ops::Shr;
use url::Url;

extern crate curl;
use std::io::{stdout, Write};
use self::curl::easy::Easy;

enum BotCommand {
    Reply
}

struct BotAction {
    command: BotCommand,
    message: Option<String>,
    from_status: Status,
    from_account: Account,
    get_url: Option<String>

}

pub fn exec(mastodon: &Mastodon) -> () {

    // println!("{:?}", mastodon.get_home_timeline().unwrap());

    let (tx, rx) = channel();
    let tx_1 = tx.clone();
    let tx_2 = tx.clone();

    for status in mastodon.get_home_timeline().unwrap() {
        let status: Status = status;
        println!("{:?}", status.content);
    }

    let (srx, nrx) = mastodon.get_user_streaming();


    let one_sec = time::Duration::from_millis(1000);

    // Status queue
    thread::spawn(move || {
        loop {
            match srx.recv() {
                Ok(status) => {
                    let status: Status = status;
                    println!("{:?}", status);
                }
                _ => {}
            }
        }
    });

    // Notification queue
    thread::spawn(move || {
        loop {
            match nrx.recv() {
                Ok(notification) => {
                    let notification: Notification = notification;
                    match notification.notification_type {
                        NotificationType::Mention => {
                            if notification.status.as_ref().unwrap().content.contains("そば") {
                                tx_1.send(BotAction {
                                    command: BotCommand::Reply,
                                    message: Some("うどん".to_owned()),
                                    from_status: notification.status.clone().unwrap(),
                                    from_account: notification.account.clone(),
                                    get_url: None
                                });
                            } else {
                              match true {
                                _ if notification.status.as_ref().unwrap().content.contains("電気けして") || notification.status.as_ref().unwrap().content.contains("電気消して") => {
                                    tx_1.send(BotAction {
                                        command: BotCommand::Reply,
                                        message: Some("あいよ".to_owned()),
                                        from_status: notification.status.clone().unwrap(),
                                        from_account: notification.account.clone(),
                                        get_url: Some("http://localhost:1880/command/light-off".to_owned())
                                    });

                                },
                                _ if notification.status.as_ref().unwrap().content.contains("電気つけて") || notification.status.as_ref().unwrap().content.contains("電気点けて") => {
                                    tx_1.send(BotAction {
                                        command: BotCommand::Reply,
                                        message: Some("あいよ".to_owned()),
                                        from_status: notification.status.clone().unwrap(),
                                        from_account: notification.account.clone(),
                                        get_url: Some("http://localhost:1880/command/light-on".to_owned())
                                    });

                                }
                                _ => {}
                              }
                            }


                        }
                        _ => {}
                    }

                    println!("{:?}", notification);
                }
                _ => {}
            }
        }
    });


    // Mastodon queue
    loop {
        match rx.recv() {
            Ok(action) => {
                let action: BotAction = action;

                let user_name = action.from_account.username.clone();
                let uri = action.from_account.url.clone();
                let domain = Url::parse(&uri).unwrap();
                let domain = domain.host_str().unwrap();
                let user_id = format!("@{}@{}", user_name, domain);
                let message = action.message.unwrap_or("".to_owned());
                let get_url = action.get_url;

                match action.command {
                    BotCommand::Reply => {
                        let mut status_b = StatusBuilder::new(format!("{} {}", user_id, message));
                        status_b.in_reply_to_id = Some(action.from_status.id as u64);
                        mastodon.new_status(status_b);
                    }
                }

                get_url.and_then( |url| {
                    let mut easy = Easy::new();
                    easy.url(&url).and_then( |_| easy.perform() );
                    Some(true)
                });
            }
            _ => {}
        }
    }
}
