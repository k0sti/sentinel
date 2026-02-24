import { useState, useEffect, useCallback } from 'react'
import { Block, Button, Preloader } from 'konsta/react'
import { useAccounts } from 'applesauce-react/hooks'
import { MapContainer, TileLayer, Marker, Popup, useMap } from 'react-leaflet'
import { trackingService } from '../services/trackingService'

function MapUpdater({ lat, lon }: { lat: number; lon: number }) {
  const map = useMap()
  useEffect(() => {
    map.setView([lat, lon], map.getZoom())
  }, [map, lat, lon])
  return null
}

export default function HomePage() {
  const accountsList = useAccounts()
  const [tracking, setTracking] = useState(false)
  const [position, setPosition] = useState<{ lat: number; lon: number } | null>(null)
  const [lastPublish, setLastPublish] = useState<number | null>(null)

  useEffect(() => {
    const sub1 = trackingService.tracking$.subscribe(setTracking)
    const sub2 = trackingService.lastPosition$.subscribe(pos => {
      if (pos) setPosition({ lat: pos.coords.latitude, lon: pos.coords.longitude })
    })
    const sub3 = trackingService.lastPublishTime$.subscribe(setLastPublish)
    return () => { sub1.unsubscribe(); sub2.unsubscribe(); sub3.unsubscribe() }
  }, [])

  const handleToggle = useCallback(async () => {
    if (tracking) {
      await trackingService.stop()
    } else {
      const account = accountsList[0]
      if (!account) {
        alert('Please set up an identity first')
        return
      }
      // Use the account's signer
      const signerFn = account.signer
        ? async (template: any) => {
            const event = await account.signer.signEvent(template)
            return event
          }
        : undefined

      await trackingService.start(undefined, signerFn)
    }
  }, [tracking, accountsList])

  const defaultCenter: [number, number] = position
    ? [position.lat, position.lon]
    : [60.17, 24.94] // Helsinki

  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: 'calc(100% - 100px)' }}>
      <div style={{ flex: 1, position: 'relative' }}>
        <MapContainer
          center={defaultCenter}
          zoom={13}
          style={{ width: '100%', height: '100%' }}
        >
          <TileLayer
            attribution='&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a>'
            url="https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png"
          />
          {position && (
            <>
              <Marker position={[position.lat, position.lon]}>
                <Popup>Your location</Popup>
              </Marker>
              <MapUpdater lat={position.lat} lon={position.lon} />
            </>
          )}
        </MapContainer>
      </div>

      <Block strong inset className="space-y-2" style={{ margin: '8px', padding: '12px' }}>
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <div>
            <strong>{tracking ? 'Tracking Active' : 'Tracking Stopped'}</strong>
            {lastPublish && (
              <div style={{ fontSize: '0.8em', opacity: 0.7 }}>
                Last update: {new Date(lastPublish).toLocaleTimeString()}
              </div>
            )}
          </div>
          <Button
            rounded
            large
            onClick={handleToggle}
            colors={{
              fillBgIos: tracking ? 'bg-red-500' : 'bg-green-500',
              fillBgMaterial: tracking ? 'bg-red-500' : 'bg-green-500',
            }}
          >
            {tracking ? 'Stop' : 'Start'}
          </Button>
        </div>
      </Block>
    </div>
  )
}
