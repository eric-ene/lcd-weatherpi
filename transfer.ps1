$main_file = "LCDWeather"
$main_file_new_name = "lcdweather"
$main_location = "target/armv7-unknown-linux-gnueabihf/release"

$dest_addr = "pi@ericpi.local"
$dest_folder = "~/projects/lcd_weather"

$additional_dirs = @()
$additional_files = @()

scp.exe "${main_location}/${main_file}" "${dest_addr}:${dest_folder}/${main_file_new_name}"

foreach ($dir in $additional_dirs) {
    scp.exe -r "${dir}" "${dest_addr}:${dest_folder}/"
}

foreach ($file in $additional_files) {
    $parent_folder = Split-Path -Parent $file
    ssh $dest_addr "mkdir -p ${dest_folder}/${parent_folder}"

    scp.exe "${file}" "${dest_addr}:${dest_folder}/${file}"
}

Write-Output "Done!"