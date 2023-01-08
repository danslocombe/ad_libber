rustup run stable-i686-pc-windows-gnu cargo build --release

set EXT_PATH=C:\Users\dan\Documents\GameMaker\Projects\efbbf_cute.gmx\extensions
set DROP_PATH=C:\Users\dan\ad_libber\target\release

set GMX_XML_NAME=ad_libber.xml
set GMX_NAME=ad_libber.extension.gmx
set DLL_NAME=ad_libber.dll
set TMP_PATH=C:\users\dan\tmp

del "%EXT_PATH%\%GMX_NAME%"
del "%EXT_PATH%\ad_libber\%DLL_NAME%"
copy "%DROP_PATH%\%DLL_NAME%" "%EXT_PATH%\ad_libber"
move "%TMP_PATH%\%GMX_XML_NAME%" "%TMP_PATH%\%GMX_NAME%"
copy "%TMP_PATH%\%GMX_NAME%" "%EXT_PATH%"