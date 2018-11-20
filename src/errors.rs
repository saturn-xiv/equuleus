error_chain!{
    foreign_links {
        StdIo(std::io::Error);
        StdStrUtf8(std::str::Utf8Error);
        StdStringFromUtf8(std::string::FromUtf8Error);

        SerialPort(serialport::Error);
    }
}
