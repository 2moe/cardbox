# Note: This is not a CLI binary, but a library that gets called, so no need to add a shebang.
require 'json'

# Decoded JSON Hash data memoization.
class JsonData
  def self.command
    @command_file ||= File.expand_path('command.json', __dir__)
    @command_hash ||= JSON.load_file(@command_file).freeze
    @command_hash
  end
end

def get_command(key, raise_err: true)
  JsonData.command[key]
    .tap { raise "command_arr is nil" if _1.nil? && raise_err }
end

# - key: str, e.g., "wasi_targets"
def rustup_add(key)
  get_command(key).each do |target|
    %w[rustup target add].push(target).run_cmd
  end
end
