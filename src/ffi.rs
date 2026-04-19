use std::{
    ffi::{CString, c_char},
    sync::atomic::{AtomicBool, Ordering},
};

pub static RUNNING: AtomicBool = AtomicBool::new(true);

extern "C" fn handle_sigint(_sig: i32) {
    RUNNING.store(false, Ordering::SeqCst);
}

unsafe extern "C" {
    fn signal(sig: i32, handler: extern "C" fn(i32)) -> usize;
}

pub fn setup_signal_handler() {
    unsafe {
        signal(2, handle_sigint);
    }
}

unsafe extern "C" {

    /*
        <== SERVER ==>
    */

    pub fn server_event_team_created(
        team_uuid: *const c_char,
        team_name: *const c_char,
        user_uuid: *const c_char,
    ) -> i32;
    pub fn server_event_user_loaded(user_uuid: *const c_char, user_name: *const c_char) -> i32;
    pub fn server_event_user_created(user_uuid: *const c_char, user_name: *const c_char) -> i32;
    pub fn server_event_user_logged_in(user_uuid: *const c_char) -> i32;
    pub fn server_event_user_logged_out(user_uuid: *const c_char) -> i32;
    pub fn server_event_private_message_sended(
        s_uuid: *const c_char,
        r_uuid: *const c_char,
        c_body: *const c_char,
    ) -> i32;
    pub fn server_event_user_subscribed(team_uuid: *const c_char, user_uuid: *const c_char) -> i32;
    pub fn server_event_user_unsubscribed(
        team_uuid: *const c_char,
        user_uuid: *const c_char,
    ) -> i32;
    pub fn server_event_channel_created(
        team_uuid: *const c_char,
        channel_uuid: *const c_char,
        name: *const c_char,
    ) -> i32;
    pub fn server_event_thread_created(
        team_uuid: *const c_char,
        channel_uuid: *const c_char,
        thead_uuid: *const c_char,
        title: *const c_char,
        body: *const c_char,
    ) -> i32;
    pub fn server_event_reply_created(
        thread_uuid: *const c_char,
        user_uuid: *const c_char,
        reply_body: *const c_char,
    ) -> i32;

    /*
        <== CLIENT ==>
    */

    pub fn client_event_logged_in(user_uuid: *const c_char, user_name: *const c_char);
    pub fn client_event_logged_out(user_uuid: *const c_char, user_name: *const c_char);
    pub fn client_event_private_message_received(
        user_uuid: *const c_char,
        message_body: *const c_char,
    );

    pub fn client_print_users(user_uuid: *const c_char, user_name: *const c_char, user_status: i32);
    pub fn client_print_user(user_uuid: *const c_char, user_name: *const c_char, user_status: i32);
    pub fn client_private_message_print_messages(
        sender_uuid: *const c_char,
        timestamp: u64,
        message_body: *const c_char,
    );
    pub fn client_print_team_created(
        team_uuid: *const c_char,
        team_name: *const c_char,
        team_description: *const c_char,
    );
    pub fn client_print_channel_created(
        channel_uuid: *const c_char,
        channel_name: *const c_char,
        channel_description: *const c_char,
    );
    pub fn client_print_thread_created(
        thread_uuid: *const c_char,
        user_uuid: *const c_char,
        thread_timestamp: u64,
        thread_title: *const c_char,
        thread_body: *const c_char,
    );
    pub fn client_print_reply_created(
        thread_uuid: *const c_char,
        user_uuid: *const c_char,
        reply_timestamp: u64,
        reply_body: *const c_char,
    );
    pub fn client_print_subscribed(user_uuid: *const c_char, team_uuid: *const c_char);
    pub fn client_print_unsubscribed(user_uuid: *const c_char, team_uuid: *const c_char);

    pub fn client_error_unauthorized();
    pub fn client_error_already_exist();
    pub fn client_error_unknown_user(user_uuid: *const c_char);
    pub fn client_error_unknown_team(team_uuid: *const c_char);
}

macro_rules! c_str {
    ($s:expr) => {
        CString::new($s).unwrap_or_default().as_ptr()
    };
}

/*
    <== SERVER ==>
*/

pub fn call_user_loaded(uuid: &str, name: &str) {
    unsafe { server_event_user_loaded(c_str!(uuid), c_str!(name)) };
}

pub fn call_user_logged_in(uuid: &str) {
    unsafe { server_event_user_logged_in(c_str!(uuid)) };
}

pub fn call_user_created(uuid: &str, name: &str) {
    unsafe { server_event_user_created(c_str!(uuid), c_str!(name)) };
}

pub fn call_user_logged_out(uuid: &str) {
    unsafe { server_event_user_logged_out(c_str!(uuid)) };
}

pub fn call_private_message_sended(sender_uuid: &str, receiver_uuid: &str, body: &str) {
    unsafe {
        server_event_private_message_sended(
            c_str!(sender_uuid),
            c_str!(receiver_uuid),
            c_str!(body),
        )
    };
}

pub fn call_user_subscribed(team_uuid: &str, user_uuid: &str) {
    unsafe { server_event_user_subscribed(c_str!(team_uuid), c_str!(user_uuid)) };
}

pub fn call_user_unsubscribed(team_uuid: &str, user_uuid: &str) {
    unsafe { server_event_user_unsubscribed(c_str!(team_uuid), c_str!(user_uuid)) };
}

pub fn call_team_created(team_uuid: &str, name: &str, user_uuid: &str) {
    unsafe { server_event_team_created(c_str!(team_uuid), c_str!(name), c_str!(user_uuid)) };
}

pub fn call_channel_created(team_uuid: &str, channel_uuid: &str, name: &str) {
    unsafe { server_event_channel_created(c_str!(team_uuid), c_str!(channel_uuid), c_str!(name)) };
}

pub fn call_thread_created(
    channel_uuid: &str,
    thread_uuid: &str,
    user_uuid: &str,
    title: &str,
    body: &str,
) {
    unsafe {
        server_event_thread_created(
            c_str!(channel_uuid),
            c_str!(thread_uuid),
            c_str!(user_uuid),
            c_str!(title),
            c_str!(body),
        )
    };
}

pub fn call_reply_created(thread_uuid: &str, user_uuid: &str, reply_body: &str) {
    unsafe {
        server_event_reply_created(c_str!(thread_uuid), c_str!(user_uuid), c_str!(reply_body))
    };
}

/*
    <== CLIENT ==>
*/

pub fn call_client_event_logged_in(user_uuid: &str, user_name: &str) {
    unsafe { client_event_logged_in(c_str!(user_uuid), c_str!(user_name)) };
}

pub fn call_client_event_logged_out(user_uuid: &str, user_name: &str) {
    unsafe { client_event_logged_out(c_str!(user_uuid), c_str!(user_name)) };
}

pub fn call_client_event_private_message_received(user_uuid: &str, message_body: &str) {
    unsafe { client_event_private_message_received(c_str!(user_uuid), c_str!(message_body)) };
}

pub fn call_client_print_users(user_uuid: &str, user_name: &str, user_status: i32) {
    unsafe { client_print_users(c_str!(user_uuid), c_str!(user_name), user_status) };
}

pub fn call_client_print_user(user_uuid: &str, user_name: &str, user_status: i32) {
    unsafe { client_print_user(c_str!(user_uuid), c_str!(user_name), user_status) };
}

pub fn call_client_private_message_print_messages(
    sender_uuid: &str,
    timestamp: u64,
    message_body: &str,
) {
    unsafe {
        client_private_message_print_messages(c_str!(sender_uuid), timestamp, c_str!(message_body))
    };
}

pub fn call_client_print_team_created(team_uuid: &str, team_name: &str, team_description: &str) {
    unsafe {
        client_print_team_created(
            c_str!(team_uuid),
            c_str!(team_name),
            c_str!(team_description),
        )
    };
}

pub fn call_client_print_channel_created(
    channel_uuid: &str,
    channel_name: &str,
    channel_description: &str,
) {
    unsafe {
        client_print_channel_created(
            c_str!(channel_uuid),
            c_str!(channel_name),
            c_str!(channel_description),
        )
    };
}

pub fn call_client_print_thread_created(
    thread_uuid: &str,
    user_uuid: &str,
    thread_timestamp: u64,
    thread_title: &str,
    thread_body: &str,
) {
    unsafe {
        client_print_thread_created(
            c_str!(thread_uuid),
            c_str!(user_uuid),
            thread_timestamp,
            c_str!(thread_title),
            c_str!(thread_body),
        )
    };
}

pub fn call_client_print_reply_created(
    thread_uuid: &str,
    user_uuid: &str,
    reply_timestamp: u64,
    reply_body: &str,
) {
    unsafe {
        client_print_reply_created(
            c_str!(thread_uuid),
            c_str!(user_uuid),
            reply_timestamp,
            c_str!(reply_body),
        )
    };
}

pub fn call_client_print_subscribed(user_uuid: &str, team_uuid: &str) {
    unsafe { client_print_subscribed(c_str!(user_uuid), c_str!(team_uuid)) };
}

pub fn call_client_print_unsubscribed(user_uuid: &str, team_uuid: &str) {
    unsafe { client_print_unsubscribed(c_str!(user_uuid), c_str!(team_uuid)) };
}

pub fn call_client_error_unauthorized() {
    unsafe { client_error_unauthorized() };
}

pub fn call_client_error_already_exist() {
    unsafe { client_error_already_exist() };
}

pub fn call_client_error_unknown_user(user_uuid: &str) {
    unsafe { client_error_unknown_user(c_str!(user_uuid)) };
}

pub fn call_client_error_unknown_team(team_uuid: &str) {
    unsafe { client_error_unknown_team(c_str!(team_uuid)) };
}

pub fn client_event_thread_reply_received(...);
pub fn client_event_team_created(...);
pub fn client_event_channel_created(...);
pub fn client_event_thread_created(...);
pub fn client_print_teams(...);
pub fn client_team_print_channels(...);
pub fn client_channel_print_threads(...);
pub fn client_thread_print_replies(...);
pub fn client_print_team(...);
pub fn client_print_channel(...);
pub fn client_print_thread(...);
pub fn client_error_unknown_channel(...);
pub fn client_error_unknown_thread(...);
pub fn call_client_print_teams(...)
pub fn call_client_team_print_channels(...)
pub fn call_client_channel_print_threads(...)
pub fn call_client_thread_print_replies(...)
pub fn call_client_print_team(...)
pub fn call_client_print_channel(...)
pub fn call_client_print_thread(...)
pub fn call_client_error_unknown_channel(...)
pub fn call_client_error_unknown_thread(...)
