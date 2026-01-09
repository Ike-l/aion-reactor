Testing:
Flag for Tracing
$env:MIRIFLAGS="-Zmiri-disable-isolation" 
Miri since uses unsafe
cargo +nightly miri test 