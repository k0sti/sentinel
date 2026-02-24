import type { CapacitorConfig } from '@capacitor/cli';

const config: CapacitorConfig = {
  appId: 'space.atlantislabs.sentinel',
  appName: 'Sentinel',
  webDir: 'dist',
  plugins: {
    BackgroundRunner: {
      label: 'space.atlantislabs.sentinel.location',
      src: 'runners/location.js',
      event: 'updateLocation',
      repeat: true,
      interval: 60,
      autoStart: false,
    },
  },
};

export default config;
