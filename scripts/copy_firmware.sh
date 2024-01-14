dd conv=nocreat,notrunc oflag=direct bs=512 if="/home/user/Documents/SteamController/Firmware/Dev/OpenSteamControllerDevBoard.bin" of="/media/user/CRP DISABLD/firmware.bin"
dd conv=nocreat,notrunc oflag=direct bs=512 if="/home/user/Documents/SteamController/Firmware/Nintendo/OpenSteamControllerNinSwitch.bin" of="/media/user/CRP DISABLD/firmware.bin"
dd conv=nocreat,notrunc oflag=direct bs=512 if="/home/user/Documents/SteamController/Firmware/Orig/firmware.bin" of="/media/user/CRP DISABLD/firmware.bin"
dd conv=nocreat,notrunc oflag=direct bs=512 if="/home/user/Documents/SteamController/Firmware/Custom/OpenSteamController.bin" of="/media/user/CRP DISABLD/firmware.bin"

dd conv=nocreat,notrunc oflag=direct bs=512 if="/home/user/Documents/SteamController/Firmware/Downloaded/4354/0/vcf_wired_controller_d0g_55393371.bin" of="/media/user/CRP DISABLD/firmware.bin"
dd conv=nocreat,notrunc oflag=direct bs=512 if="/home/user/Documents/SteamController/Firmware/Downloaded/4612/3/firmware_4612_6041128E.bin" of="/media/user/CRP DISABLD/firmware.bin"
umount "/media/user/CRP DISABLD"

http://media.steampowered.com/controller_config/firmware/firmware_4612_6041128E.bin

LANG=C grep --only-matching --byte-offset --binary --text --perl-regexp "" <file>

vbindiff "/home/user/Documents/SteamController/Firmware/Downloaded/4354/18/vcf_wired_controller_d0g_57bf5c10.bin" "/home/user/Documents/SteamController/Firmware/Orig/firmware.bin"