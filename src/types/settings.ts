export interface AppSettings {
  /** User interface theme */
  theme: string;
  /** Whether automatic backups are enabled */
  auto_backup: boolean;
  /** Whether notifications are enabled */
  notifications_enabled: boolean;
  /** Log level setting */
  log_level: string;
  /** Developer mode enabled */
  developer_mode: boolean;  /** Whether to skip seed phrase dialogs during wallet creation */
  skip_seed_phrase_dialogs: boolean;
  /** Whether to minimize to system tray (enables system tray functionality) */
  minimize_to_system_tray: boolean;
}
