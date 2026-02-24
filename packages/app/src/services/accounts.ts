import { AccountManager } from 'applesauce-accounts'
import { registerCommonAccountTypes } from 'applesauce-accounts/accounts'
import { skip } from 'rxjs'

const STORAGE_KEY = 'sentinel-accounts'

export const accounts = new AccountManager()
registerCommonAccountTypes(accounts)

// Persist accounts to localStorage
accounts.accounts$.pipe(skip(1)).subscribe(() => {
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(accounts.toJSON()))
  } catch {
    // Ignore storage errors
  }
})

// Load persisted accounts
try {
  const stored = localStorage.getItem(STORAGE_KEY)
  if (stored) {
    const parsed = JSON.parse(stored)
    accounts.fromJSON(parsed, true)
  }
} catch {
  // Ignore parse errors
}
