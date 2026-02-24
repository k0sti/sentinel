import { useState, useEffect } from 'react'
import { Block, BlockTitle, List, ListInput, ListItem, Toggle, Button } from 'konsta/react'
import { trackingService, TrackingConfig } from '../services/trackingService'
import { relayService } from '../services/relayService'

export default function SettingsPage() {
  const [config, setConfig] = useState<TrackingConfig>(trackingService.config$.value)
  const [relays, setRelays] = useState<string>(relayService.relayUrls$.value.join('\n'))

  useEffect(() => {
    const sub = trackingService.config$.subscribe(setConfig)
    return () => sub.unsubscribe()
  }, [])

  const updateConfig = (partial: Partial<TrackingConfig>) => {
    const updated = { ...config, ...partial }
    setConfig(updated)
    trackingService.saveConfig(updated)
  }

  const saveRelays = () => {
    const urls = relays
      .split('\n')
      .map(r => r.trim())
      .filter(r => r.startsWith('wss://') || r.startsWith('ws://'))
    relayService.saveRelays(urls)
  }

  return (
    <div style={{ overflow: 'auto', height: 'calc(100% - 100px)' }}>
      <BlockTitle>Tracking</BlockTitle>
      <List strongIos>
        <ListInput
          label="Interval (seconds)"
          type="number"
          value={String(config.intervalSecs)}
          onChange={(e: any) => updateConfig({ intervalSecs: Number(e.target.value) || 60 })}
        />
        <ListInput
          label="Geohash Precision (1-12)"
          type="number"
          value={String(config.precision)}
          onChange={(e: any) => {
            const v = Math.min(12, Math.max(1, Number(e.target.value) || 8))
            updateConfig({ precision: v })
          }}
        />
        <ListInput
          label="D-Tag Identifier"
          type="text"
          value={config.dTag}
          onChange={(e: any) => updateConfig({ dTag: e.target.value || 'default' })}
        />
        <ListInput
          label="Expiration TTL (seconds)"
          type="number"
          value={String(config.expirationSecs)}
          onChange={(e: any) => updateConfig({ expirationSecs: Number(e.target.value) || 3600 })}
        />
      </List>

      <BlockTitle>Encryption</BlockTitle>
      <List strongIos>
        <ListItem
          label
          title="Encrypt Location"
          after={
            <Toggle
              checked={config.encrypted}
              onChange={() => updateConfig({ encrypted: !config.encrypted })}
            />
          }
        />
        {config.encrypted && (
          <ListInput
            label="Recipient Pubkeys (hex, one per line)"
            type="textarea"
            value={config.recipientPubkeys.join('\n')}
            onChange={(e: any) =>
              updateConfig({
                recipientPubkeys: e.target.value
                  .split('\n')
                  .map((s: string) => s.trim())
                  .filter(Boolean),
              })
            }
          />
        )}
      </List>

      <BlockTitle>Relays</BlockTitle>
      <List strongIos>
        <ListInput
          label="Relay URLs (one per line)"
          type="textarea"
          value={relays}
          onChange={(e: any) => setRelays(e.target.value)}
        />
      </List>
      <Block>
        <Button onClick={saveRelays}>Save Relays</Button>
      </Block>
    </div>
  )
}
