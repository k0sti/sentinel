import { BehaviorSubject } from 'rxjs'
import { Position } from '@capacitor/geolocation'
import { EventTemplate, finalizeEvent, nip44, getPublicKey } from 'nostr-tools'
import { locationService } from './locationService'
import { relayService } from './relayService'

export interface TrackingConfig {
  intervalSecs: number
  precision: number
  encrypted: boolean
  recipientPubkeys: string[]
  dTag: string
  expirationSecs: number
}

const DEFAULT_CONFIG: TrackingConfig = {
  intervalSecs: 60,
  precision: 8,
  encrypted: false,
  recipientPubkeys: [],
  dTag: 'default',
  expirationSecs: 3600,
}

class TrackingService {
  tracking$ = new BehaviorSubject(false)
  lastPosition$ = new BehaviorSubject<Position | null>(null)
  lastPublishTime$ = new BehaviorSubject<number | null>(null)
  config$ = new BehaviorSubject<TrackingConfig>(this.loadConfig())

  private intervalId: number | null = null
  private lastGeohash: string | null = null

  private loadConfig(): TrackingConfig {
    try {
      const stored = localStorage.getItem('sentinel-tracking-config')
      if (stored) return { ...DEFAULT_CONFIG, ...JSON.parse(stored) }
    } catch { /* ignore */ }
    return DEFAULT_CONFIG
  }

  saveConfig(config: TrackingConfig) {
    localStorage.setItem('sentinel-tracking-config', JSON.stringify(config))
    this.config$.next(config)
  }

  async start(secretKeyHex?: string, signerFn?: (event: EventTemplate) => Promise<any>) {
    if (this.tracking$.value) return

    this.tracking$.next(true)

    const callback = async (position: Position | null) => {
      if (!position) return
      this.lastPosition$.next(position)
    }

    await locationService.startWatching(callback)

    // Periodic publish
    this.intervalId = window.setInterval(async () => {
      const pos = this.lastPosition$.value
      if (!pos) return

      try {
        await this.publishLocation(pos, secretKeyHex, signerFn)
      } catch (e) {
        console.error('Failed to publish location:', e)
      }
    }, this.config$.value.intervalSecs * 1000)
  }

  async stop() {
    if (!this.tracking$.value) return

    this.tracking$.next(false)
    await locationService.stopWatching()

    if (this.intervalId !== null) {
      clearInterval(this.intervalId)
      this.intervalId = null
    }
  }

  private async publishLocation(
    position: Position,
    secretKeyHex?: string,
    signerFn?: (event: EventTemplate) => Promise<any>,
  ) {
    const config = this.config$.value
    const { latitude, longitude, accuracy } = position.coords
    const expiration = Math.floor(Date.now() / 1000) + config.expirationSecs

    // Encode geohash using simple algorithm (or WASM if loaded)
    const geohash = encodeGeohash(latitude, longitude, config.precision)

    if (!config.encrypted) {
      // Kind 30472 — public
      const tags: string[][] = [
        ['g', geohash],
        ['d', config.dTag],
        ['expiration', String(expiration)],
      ]
      if (accuracy !== null && accuracy !== undefined) {
        tags.push(['accuracy', String(accuracy)])
      }

      const template: EventTemplate = {
        kind: 30472,
        created_at: Math.floor(Date.now() / 1000),
        tags,
        content: '',
      }

      let signed: any
      if (signerFn) {
        signed = await signerFn(template)
      } else if (secretKeyHex) {
        const sk = hexToBytes(secretKeyHex)
        signed = finalizeEvent(template, sk)
      } else {
        console.warn('No signer available')
        return
      }

      await relayService.publish(signed)
      this.lastPublishTime$.next(Date.now())
    } else {
      // Kind 30473 — encrypted
      for (const recipientPubkey of config.recipientPubkeys) {
        const payload = JSON.stringify([
          ['g', geohash],
          ...(accuracy != null ? [['accuracy', String(accuracy)]] : []),
        ])

        if (!secretKeyHex) {
          console.warn('Encrypted mode requires nsec for NIP-44')
          return
        }

        const sk = hexToBytes(secretKeyHex)
        const encrypted = nip44.v2.encrypt(payload, nip44.v2.utils.getConversationKey(sk, recipientPubkey))

        const tags: string[][] = [
          ['p', recipientPubkey],
          ['d', config.dTag],
          ['expiration', String(expiration)],
        ]

        const template: EventTemplate = {
          kind: 30473,
          created_at: Math.floor(Date.now() / 1000),
          tags,
          content: encrypted,
        }

        let signed: any
        if (signerFn) {
          signed = await signerFn(template)
        } else {
          signed = finalizeEvent(template, sk)
        }

        await relayService.publish(signed)
        this.lastPublishTime$.next(Date.now())
      }
    }
  }
}

// Simple geohash encoder (avoids WASM dependency for basic operation)
function encodeGeohash(lat: number, lon: number, precision: number): string {
  const BASE32 = '0123456789bcdefghjkmnpqrstuvwxyz'
  let idx = 0
  let bit = 0
  let evenBit = true
  let hash = ''
  let latMin = -90, latMax = 90
  let lonMin = -180, lonMax = 180

  while (hash.length < precision) {
    if (evenBit) {
      const mid = (lonMin + lonMax) / 2
      if (lon >= mid) { idx = idx * 2 + 1; lonMin = mid }
      else { idx = idx * 2; lonMax = mid }
    } else {
      const mid = (latMin + latMax) / 2
      if (lat >= mid) { idx = idx * 2 + 1; latMin = mid }
      else { idx = idx * 2; latMax = mid }
    }
    evenBit = !evenBit
    if (++bit === 5) {
      hash += BASE32[idx]
      bit = 0
      idx = 0
    }
  }
  return hash
}

function hexToBytes(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2)
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16)
  }
  return bytes
}

export const trackingService = new TrackingService()
