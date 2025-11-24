#[path = "output_rust/example.rs"]
mod example;

use example::*;

fn main() {
    let mut buf: UserBuffer = [
        8, 7, 6, 5, 4, 3, 2, 1,    // id (u64)
        0x89, // name/yes (u8 bit-field)
        13, 12, 11, 10, // email (u32)
        14, 16, 15, 1, // hdr (Header, 4 packed bytes)
    ];

    let user_view = UserView::new(&buf);

    assert_eq!(user_view.id(), 578437695752307201);
    assert_eq!(user_view.name(), 9);
    assert_eq!(user_view.yes(), true);
    assert_eq!(user_view.email(), 218893066);
    assert_eq!(user_view.hdr().version(), 14);
    assert_eq!(user_view.hdr().checksum(), 4111);
    assert_eq!(user_view.hdr().tag(), Status::Active);

    let mut user_mut_view = UserMutView::new(&mut buf);

    assert_eq!(user_mut_view.id(), 578437695752307201);
    assert_eq!(user_mut_view.name(), 9);
    assert_eq!(user_mut_view.yes(), true);
    assert_eq!(user_mut_view.email(), 218893066);
    assert_eq!(user_mut_view.hdr().version(), 14);
    assert_eq!(user_mut_view.hdr_mut().version(), 14);
    assert_eq!(user_mut_view.hdr().checksum(), 4111);
    assert_eq!(user_mut_view.hdr_mut().checksum(), 4111);
    assert_eq!(user_mut_view.hdr().tag(), Status::Active);
    assert_eq!(user_mut_view.hdr_mut().tag(), Status::Active);

    user_mut_view.set_id(1681321687);
    user_mut_view.set_name(2);
    user_mut_view.set_yes(false);
    user_mut_view.set_email(34764);
    user_mut_view.hdr_mut().set_version(0);
    user_mut_view.hdr_mut().set_checksum(300);
    user_mut_view.hdr_mut().set_tag(Status::Error);

    assert_eq!(user_mut_view.id(), 1681321687);
    assert_eq!(user_mut_view.name(), 2);
    assert_eq!(user_mut_view.yes(), false);
    assert_eq!(user_mut_view.email(), 34764);
    assert_eq!(user_mut_view.hdr().version(), 0);
    assert_eq!(user_mut_view.hdr_mut().version(), 0);
    assert_eq!(user_mut_view.hdr().checksum(), 300);
    assert_eq!(user_mut_view.hdr_mut().checksum(), 300);
    assert_eq!(user_mut_view.hdr().tag(), Status::Error);
    assert_eq!(user_mut_view.hdr_mut().tag(), Status::Error);
}
