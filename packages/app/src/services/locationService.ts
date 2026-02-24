import { Geolocation, Position } from '@capacitor/geolocation'
import { Capacitor } from '@capacitor/core'

type PositionCallback = (position: Position | null) => void

class LocationService {
  private callbacks = new Set<PositionCallback>()
  private watchId: string | null = null

  async startWatching(callback: PositionCallback) {
    this.callbacks.add(callback)
    if (this.watchId !== null) return

    if (!Capacitor.isNativePlatform()) {
      return this.startWebWatch()
    }

    this.watchId = await Geolocation.watchPosition(
      { enableHighAccuracy: true, maximumAge: 0 },
      (position, error) => {
        this.callbacks.forEach(cb => cb(error ? null : position))
      },
    )
  }

  private startWebWatch() {
    if (!navigator.geolocation) return

    const id = navigator.geolocation.watchPosition(
      (pos) => {
        const position: Position = {
          timestamp: pos.timestamp,
          coords: {
            latitude: pos.coords.latitude,
            longitude: pos.coords.longitude,
            accuracy: pos.coords.accuracy,
            altitude: pos.coords.altitude,
            altitudeAccuracy: pos.coords.altitudeAccuracy,
            heading: pos.coords.heading,
            speed: pos.coords.speed,
          },
        }
        this.callbacks.forEach(cb => cb(position))
      },
      () => {
        this.callbacks.forEach(cb => cb(null))
      },
      { enableHighAccuracy: true, maximumAge: 0 },
    )
    this.watchId = String(id)
  }

  async stopWatching(callback?: PositionCallback) {
    if (callback) {
      this.callbacks.delete(callback)
    } else {
      this.callbacks.clear()
    }

    if (this.callbacks.size > 0) return

    if (this.watchId !== null) {
      if (Capacitor.isNativePlatform()) {
        await Geolocation.clearWatch({ id: this.watchId })
      } else {
        navigator.geolocation.clearWatch(Number(this.watchId))
      }
      this.watchId = null
    }
  }
}

export const locationService = new LocationService()
