project(
  'nett-icon-viewer', 'rust',
  version: '0.1.0',
  default_options: ['warning_level=2']
)

gnome = import('gnome')

rust_dep = dependency('gtk4', required: true)
cargo = find_program('cargo', required: true)

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
datadir = prefix / get_option('datadir')

install_data('resources/NettIconViewer.desktop',
  install_dir: join_paths(get_option('datadir'), 'applications'),
  rename: 'codes.blaine.NettIconViewer.desktop',
)

subdir('icon-viewer/src')

icon_names = [
  '16x16',
  '24x24',
  '32x32',
  '48x48',
  '64x64',
  '128x128',
  '128x128@2x',
  '512x512',
  '1024x1024',
  'scalable',
]

icons = custom_target('generate-icons',
  output: icon_names,
  install_dir: join_paths(get_option('datadir'), 'icons', 'hicolor'),
  install: true,
  command : [
    cargo, 'run', '--package', 'generate_icons', '--', meson.current_build_dir()
  ]
)

gnome.post_install(
  gtk_update_icon_cache: true,
  update_desktop_database: true,
)
