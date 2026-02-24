import { RelayPool } from 'applesauce-relay'
import { BehaviorSubject } from 'rxjs'

const DEFAULT_RELAYS = ['wss://zooid.atlantislabs.space']

class RelayService {
  pool = new RelayPool()
  relayUrls$ = new BehaviorSubject<string[]>(this.loadRelays())

  private loadRelays(): string[] {
    try {
      const stored = localStorage.getItem('sentinel-relays')
      if (stored) return JSON.parse(stored)
    } catch { /* ignore */ }
    return DEFAULT_RELAYS
  }

  saveRelays(urls: string[]) {
    localStorage.setItem('sentinel-relays', JSON.stringify(urls))
    this.relayUrls$.next(urls)
  }

  getGroup() {
    return this.pool.group(this.relayUrls$.value)
  }

  async publish(event: any) {
    return this.pool.publish(this.relayUrls$.value, event)
  }
}

export const relayService = new RelayService()
