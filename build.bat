set OPENSSL_DIR=%CD%\openssl_3.5.2_arm64-v8a
set OPENSSL_LIB=%CD%\openssl_3.5.2_arm64-v8a\lib
set OPENSSL_INCLUDE=%CD%\openssl_3.5.2_arm64-v8a\include
cargo tauri android build --apk --target aarch64