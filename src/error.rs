use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    ChanRecv(::std::sync::mpsc::RecvError),
    ChanSend(::std::sync::mpsc::SendError<::websocket::Message<'static>>),
    Http(::reqwest::Error),
    Io(::std::io::Error),
    Json(::serde_json::Error),
    Socket(::websocket::result::WebSocketError),
    Toml(::toml::de::Error),
    Url(::websocket::url::ParseError)
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ChanRecv(ref e) => fmt::Display::fmt(e, f),
            Error::ChanSend(ref e) => fmt::Display::fmt(e, f),
            Error::Http(ref e) => fmt::Display::fmt(e, f),
            Error::Io(ref e) => fmt::Display::fmt(e, f),
            Error::Json(ref e) => fmt::Display::fmt(e, f),
            Error::Socket(ref e) => fmt::Display::fmt(e, f),
            Error::Toml(ref e) => fmt::Display::fmt(e, f),
            Error::Url(ref e) => fmt::Display::fmt(e, f)
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ChanRecv(ref e) => e.description(),
            Error::ChanSend(ref e) => e.description(),
            Error::Http(ref e) => e.description(),
            Error::Io(ref e) => e.description(),
            Error::Json(ref e) => e.description(),
            Error::Socket(ref e) => e.description(),
            Error::Toml(ref e) => e.description(),
            Error::Url(ref e) => e.description()
        }
    }

    fn cause(&self) -> Option<&StdError> {
        match *self {
            Error::ChanRecv(ref e) => Some(e),
            Error::ChanSend(ref e) => Some(e),
            Error::Http(ref e) => Some(e),
            Error::Io(ref e) => Some(e),
            Error::Json(ref e) => Some(e),
            Error::Socket(ref e) => Some(e),
            Error::Toml(ref e) => Some(e),
            Error::Url(ref e) => Some(e)
        }
    }
}

impl From<::std::sync::mpsc::RecvError> for Error {
    fn from (err: ::std::sync::mpsc::RecvError) -> Error {
        Error::ChanRecv(err)
    }
}

impl From<::std::sync::mpsc::SendError<::websocket::Message<'static>>> for Error {
    fn from (err: ::std::sync::mpsc::SendError<::websocket::Message<'static>>) -> Error {
        Error::ChanSend(err)
    }
}

impl From<::reqwest::Error> for Error {
    fn from (err: ::reqwest::Error) -> Error {
        Error::Http(err)
    }
}

impl From<::std::io::Error> for Error {
    fn from (err: ::std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<::serde_json::Error> for Error {
    fn from (err: ::serde_json::Error) -> Error {
        Error::Json(err)
    }
}

impl From<::websocket::url::ParseError> for Error {
    fn from(err: ::websocket::url::ParseError) -> Error {
        Error::Url(err)
    }
}

impl From<::toml::de::Error> for Error {
    fn from(err: ::toml::de::Error) -> Error {
        Error::Toml(err)
    }
}

impl From<::websocket::result::WebSocketError> for Error {
    fn from(err: ::websocket::result::WebSocketError) -> Error {
        Error::Socket(err)
    }
}

pub type Result<T> = ::std::result::Result<T, Error>;
