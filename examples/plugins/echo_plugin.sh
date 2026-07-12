#!/data/data/com.termux/files/usr/bin/bash
if [ "$1" == "--orbvynx-manifest" ]; then
  echo '{"name":"echo_plugin","version":"1.0.0","description":"Echoes input back","capability_name":"plugin.echo"}'
  exit 0
fi

if [ "$1" == "--orbvynx-invoke" ]; then
  input=$(cat)
  echo "{\"received\": $input}"
  exit 0
fi

exit 1
