#!/usr/bin/env -S ruby --disable=gems
# ===================================
require 'json'

def get_command(key, raise_err: true)
  File
    .expand_path('command.json', __dir__)
    .then {|p| JSON.load_file p}
    .then {|json| json[key]}
    .tap { raise "command_arr is nil" if _1.nil? && raise_err }
end

# command_arr.run_cmd
