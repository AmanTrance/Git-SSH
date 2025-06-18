# REMOTE GIT SERVER (ONLY SSH)

## ` Supports All Remote Git Operations `

* NOTE: CREATION BARE GIT REPO ON REMOTE SERVER NEEDS TO BE HANDLED \
    BY SOME OTHER SERVICE (RECOMMENDED) \
    OR MANUALLY (IF YOU JUST WANT TO TEST)

### STEPS TO BUILD (for linux/amd64)

* rustup target add x86_64-unknown-linux-musl
* cargo build --release --target x86_64-unknown-linux-musl
* strip ./path/to/binary (it will be available inside the target directory)