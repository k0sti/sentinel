import { useState } from 'react'
import { App as KonstaApp, Page, Navbar, Tabbar, TabbarLink } from 'konsta/react'
import { AccountsProvider } from 'applesauce-react'
import { accounts } from './services/accounts'
import HomePage from './pages/HomePage'
import SettingsPage from './pages/SettingsPage'
import IdentityPage from './pages/IdentityPage'

type Tab = 'home' | 'settings' | 'identity'

export default function App() {
  const [activeTab, setActiveTab] = useState<Tab>('home')

  return (
    <AccountsProvider manager={accounts}>
      <KonstaApp theme="ios" safeAreas>
        <Page>
          <Navbar title="Sentinel" />

          {activeTab === 'home' && <HomePage />}
          {activeTab === 'settings' && <SettingsPage />}
          {activeTab === 'identity' && <IdentityPage />}

          <Tabbar>
            <TabbarLink
              active={activeTab === 'home'}
              onClick={() => setActiveTab('home')}
              label="Map"
            />
            <TabbarLink
              active={activeTab === 'settings'}
              onClick={() => setActiveTab('settings')}
              label="Settings"
            />
            <TabbarLink
              active={activeTab === 'identity'}
              onClick={() => setActiveTab('identity')}
              label="Identity"
            />
          </Tabbar>
        </Page>
      </KonstaApp>
    </AccountsProvider>
  )
}
