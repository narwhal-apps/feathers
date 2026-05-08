declare global {
  namespace App {
    interface ImportMetaEnv {
      readonly VITE_APP_VERSION: string;
      readonly VITE_BUILD_SHA: string;
    }
  }
}

interface ImportMetaEnv {
  readonly VITE_APP_VERSION: string;
  readonly VITE_BUILD_SHA: string;
}

export {};
