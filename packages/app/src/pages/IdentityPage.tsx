import { useState, useCallback } from 'react'
import { Block, BlockTitle, List, ListInput, ListItem, Button } from 'konsta/react'
import { useAccountManager, useAccounts } from 'applesauce-react/hooks'
import { SimpleAccount, ExtensionAccount } from 'applesauce-accounts/accounts'
import { SimpleSigner, ExtensionSigner } from 'applesauce-signers'
import { Capacitor } from '@capacitor/core'
import { getPublicKey, nip19 } from 'nostr-tools'

function hexToBytes(hex: string): Uint8Array {
  const bytes = new Uint8Array(hex.length / 2)
  for (let i = 0; i < hex.length; i += 2) {
    bytes[i / 2] = parseInt(hex.substring(i, i + 2), 16)
  }
  return bytes
}

function bytesToHex(bytes: Uint8Array): string {
  return Array.from(bytes).map(b => b.toString(16).padStart(2, '0')).join('')
}

export default function IdentityPage() {
  const manager = useAccountManager()
  const accountsList = useAccounts()
  const [nsecInput, setNsecInput] = useState('')

  const addNsec = useCallback(() => {
    try {
      let secretKeyHex: string
      if (nsecInput.startsWith('nsec')) {
        const decoded = nip19.decode(nsecInput)
        if (decoded.type !== 'nsec') throw new Error('Invalid nsec')
        secretKeyHex = bytesToHex(decoded.data)
      } else {
        secretKeyHex = nsecInput
      }

      const sk = hexToBytes(secretKeyHex)
      const pubkey = getPublicKey(sk)
      const signer = new SimpleSigner(sk)
      const account = new SimpleAccount(pubkey, signer)
      manager.addAccount(account)
      setNsecInput('')
    } catch (e: any) {
      alert('Invalid nsec: ' + e.message)
    }
  }, [nsecInput, manager])

  const addExtension = useCallback(async () => {
    try {
      const signer = new ExtensionSigner()
      const pubkey = await signer.getPublicKey()
      const account = new ExtensionAccount(pubkey, signer)
      manager.addAccount(account)
    } catch (e: any) {
      alert('NIP-07 extension not found: ' + e.message)
    }
  }, [manager])

  const removeAccount = useCallback((account: any) => {
    manager.removeAccount(account)
  }, [manager])

  const isWeb = !Capacitor.isNativePlatform()

  return (
    <div style={{ overflow: 'auto', height: 'calc(100% - 100px)' }}>
      <BlockTitle>Current Identity</BlockTitle>
      {accountsList.length === 0 ? (
        <Block>
          <p>No identity configured. Add one below.</p>
        </Block>
      ) : (
        <List strongIos>
          {accountsList.map((account, i) => (
            <ListItem
              key={i}
              title={account.pubkey.slice(0, 16) + '...'}
              subtitle={account.type || 'unknown'}
              after={
                <Button small tonal onClick={() => removeAccount(account)}>
                  Remove
                </Button>
              }
            />
          ))}
        </List>
      )}

      <BlockTitle>Add Identity</BlockTitle>
      <List strongIos>
        <ListInput
          label="nsec or hex secret key"
          type="password"
          value={nsecInput}
          onChange={(e: any) => setNsecInput(e.target.value)}
          placeholder="nsec1... or hex"
        />
      </List>
      <Block style={{ display: 'flex', gap: '8px', flexWrap: 'wrap' }}>
        <Button onClick={addNsec}>Add nsec</Button>
        {isWeb && (
          <Button tonal onClick={addExtension}>
            NIP-07 Extension
          </Button>
        )}
      </Block>

      {!isWeb && (
        <>
          <BlockTitle>Android</BlockTitle>
          <Block>
            <p style={{ fontSize: '0.9em', opacity: 0.7 }}>
              Amber signer support: use the nsec import above or connect via NIP-46.
            </p>
          </Block>
        </>
      )}
    </div>
  )
}
