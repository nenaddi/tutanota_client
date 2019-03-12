// Copyright 2019 Fredrik Portström <https://portstrom.com>
// This is free software distributed under the terms specified in
// the file LICENSE at the top-level directory of this distribution.

use futures::{
    future::{self, Either},
    Future, Stream,
};

fn main() {
    let mut arguments = std::env::args();
    if arguments.len() != 2 {
        eprintln!("Usage: {} email_address", arguments.next().unwrap());
        std::process::exit(1);
    }
    let password = rpassword::prompt_password_stderr("Password: ").unwrap_or_else(|error| {
        eprintln!("Failed to read password: {}", error);
        std::process::exit(1);
    });
    let email_address = arguments.nth(1).unwrap();
    hyper::rt::run(hyper::rt::lazy(|| {
        let https = hyper_tls::HttpsConnector::new(4).unwrap();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);
        tutanota_client::salt::fetch_salt(&client, &email_address)
            .and_then(move |salt| {
                let user_passphrase_key =
                    tutanota_client::create_user_passphrase_key(&password, &salt);
                tutanota_client::session::fetch_session(
                    &client,
                    "Rust",
                    &email_address,
                    &user_passphrase_key,
                )
                .and_then(move |response| {
                    let access_token = response.access_token;
                    tutanota_client::user::fetch_user(&client, &access_token, &response.user)
                        .and_then(move |response| {
                            // XXX avoid panic
                            let membership = response
                                .memberships
                                .iter()
                                .find(|membership| membership.group_type == "5")
                                .unwrap();
                            // XXX avoid panic
                            let user_group_key = tutanota_client::decrypt_key(
                                &user_passphrase_key,
                                &response.user_group.sym_enc_g_key,
                            )
                            .unwrap();
                            // XXX avoid panic
                            let group_key = tutanota_client::decrypt_key(
                                &user_group_key,
                                &membership.sym_enc_g_key,
                            )
                            .unwrap();
                            tutanota_client::mailboxgrouproot::fetch_mailboxgrouproot(
                                &client,
                                &access_token,
                                &membership.group,
                            )
                            .and_then(move |mailbox| {
                                tutanota_client::mailbox::fetch_mailbox(
                                    &client,
                                    &access_token,
                                    &mailbox,
                                )
                                .and_then(move |folders| {
                                    tutanota_client::mailfolder::fetch_mailfolder(
                                        &client,
                                        &access_token,
                                        &folders,
                                    )
                                    .and_then(move |folders| {
                                        // XXX avoid panic
                                        tutanota_client::mail::fetch_mail(
                                            &client,
                                            &access_token,
                                            &folders[0].mails,
                                        )
                                        .and_then(
                                            move |mails| {
                                                for mail in &mails {
                                                    // XXX avoid panic
                                                    let session_key = tutanota_client::decrypt_key(
                                                        &group_key,
                                                        &mail.owner_enc_session_key,
                                                    )
                                                    .unwrap();
                                                    let session_sub_keys =
                                                        tutanota_client::SubKeys::new(session_key);
                                                    // XXX avoid panic
                                                    let title = tutanota_client::decrypt_with_mac(
                                                        &session_sub_keys,
                                                        &mail.subject,
                                                    )
                                                    .unwrap();
                                                    // XXX avoid panic
                                                    println!(
                                                        "mail, subject: {:?}, from: {:?}",
                                                        std::str::from_utf8(&title).unwrap(),
                                                        mail.sender.address,
                                                    );
                                                }
                                                // XXX avoid panic
                                                let mail = mails.into_iter().next().unwrap();
                                                fetch_mail(client, access_token, group_key, mail)
                                            },
                                        )
                                    })
                                })
                            })
                        })
                })
            })
            .or_else(|error| {
                eprintln!("Error: {:#?}", error);
                match error {
                    tutanota_client::Error::ContentType(response)
                    | tutanota_client::Error::Status(response) => {
                        Either::A(response.into_body().concat2().then(|result| {
                            match result {
                                Err(error) => eprintln!("Network error: {}", error),
                                Ok(response_body) => {
                                    eprintln!(
                                        "Response body: {:?}",
                                        std::str::from_utf8(&response_body)
                                    );
                                }
                            }
                            Ok(())
                        }))
                    }
                    _ => Either::B(future::ok(())),
                }
            })
    }));
}

fn fetch_mail<C: 'static + hyper::client::connect::Connect>(
    client: hyper::Client<C, hyper::Body>,
    access_token: String,
    group_key: [u8; 16],
    mail: tutanota_client::mail::Mail,
) -> impl Future<Error = tutanota_client::Error, Item = ()> {
    let attachment_future = match mail.attachments.first() {
        None => Either::A(future::ok(None)),
        Some(attachment) => {
            let file_future = tutanota_client::file::fetch_file(&client, &access_token, attachment);
            let filedata_future =
                tutanota_client::filedata::fetch_filedata(&client, &access_token, attachment)
                    .and_then(|response| {
                        response.concat2().map_err(tutanota_client::Error::Network)
                    });
            Either::B(file_future.join(filedata_future).map(Some))
        }
    };
    let mailbody_future =
        tutanota_client::mailbody::fetch_mailbody(&client, &access_token, &mail.body);
    let session_key = mail.owner_enc_session_key;
    attachment_future
        .join(mailbody_future)
        .map(move |(file, text)| {
            // XXX avoid panic
            let session_key = tutanota_client::decrypt_key(&group_key, &session_key).unwrap();
            let session_sub_keys = tutanota_client::SubKeys::new(session_key);
            // XXX avoid panic
            let text = tutanota_client::decrypt_with_mac(&session_sub_keys, &text).unwrap();
            // XXX avoid panic
            println!("mail body: {}", std::str::from_utf8(&text).unwrap());
            if let Some((file, file_data)) = file {
                // XXX avoid panic
                let session_key =
                    tutanota_client::decrypt_key(&group_key, &file.owner_enc_session_key).unwrap();
                let session_sub_keys = tutanota_client::SubKeys::new(session_key);
                // XXX avoid panic
                let mime_type =
                    tutanota_client::decrypt_with_mac(&session_sub_keys, &file.mime_type).unwrap();
                // XXX avoid panic
                let name =
                    tutanota_client::decrypt_with_mac(&session_sub_keys, &file.name).unwrap();
                // XXX avoid panic
                println!(
                    "attachment, mime type: {:?}, name: {:?}, size: {:?}",
                    std::str::from_utf8(&mime_type).unwrap(),
                    std::str::from_utf8(&name).unwrap(),
                    file.size
                );
                // XXX avoid panic
                let file_data =
                    tutanota_client::decrypt_with_mac(&session_sub_keys, &file_data).unwrap();
                println!("file data: {:?}", std::str::from_utf8(&file_data));
            }
        })
}
