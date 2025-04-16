/**
 * Interface for application settings that matches the Rust AppSettings struct
 */
export interface AppSettings {
  /** User interface theme */
  theme: string;
  /** Whether automatic backups are enabled */
  auto_backup: boolean;
  /** Whether notifications are enabled */
  notifications_enabled: boolean;
  /** Log level setting */
  log_level: string;
  /** Whether to show seed phrase dialogs during wallet creation */
  show_seed_phrase_dialogs: boolean;
}
