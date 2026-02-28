#!/usr/bin/env ruby
# ===================================
# Depends: ruby (>= 3.1)
# RubyGems: cinnabar (>= 0.0.8)
# ===================================
# frozen_string_literal: true

require 'json'
require 'cinnabar'
using Cinnabar::Command::ArrRefin

workflow_yml = 'build_cardbox_target.yml'

# json data
stdin_data = File
  .expand_path('build.json', __dir__)
  .then { JSON.load_file _1 }
  .except('$schema') # remove schema key
  .transform_values(&:to_s) # gh cli v2.87 requires JSON values to be Strings
  .then { JSON.dump _1 }

%w[gh workflow run]
  .concat([workflow_yml, '--json'])
  .run(opts: { stdin_data: })
  .then(&:display)
