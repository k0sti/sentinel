// Background runner for @capawesome/capacitor-background-runner
// This runs in a restricted JS context (no DOM, limited APIs)

addEventListener('updateLocation', async (resolve, reject) => {
  try {
    // Background runner has limited API — dispatch to main app via CapacitorNotifications
    // The main app's tracking service handles the actual location fetch + publish
    const response = await CapacitorNotifications.schedule([{
      title: 'Sentinel',
      body: 'Updating location...',
      id: 1,
    }])

    resolve()
  } catch (e) {
    reject(e)
  }
})
