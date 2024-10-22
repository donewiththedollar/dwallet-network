import React from 'react';
import ComponentCreator from '@docusaurus/ComponentCreator';

export default [
  {
    path: '/__docusaurus/debug',
    component: ComponentCreator('/__docusaurus/debug', '9e4'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/config',
    component: ComponentCreator('/__docusaurus/debug/config', 'f55'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/content',
    component: ComponentCreator('/__docusaurus/debug/content', '14b'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/globalData',
    component: ComponentCreator('/__docusaurus/debug/globalData', 'c3f'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/metadata',
    component: ComponentCreator('/__docusaurus/debug/metadata', '7d9'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/registry',
    component: ComponentCreator('/__docusaurus/debug/registry', '9e2'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/routes',
    component: ComponentCreator('/__docusaurus/debug/routes', '5d1'),
    exact: true
  },
  {
    path: '/markdown-page',
    component: ComponentCreator('/markdown-page', '508'),
    exact: true
  },
  {
    path: '/',
    component: ComponentCreator('/', '369'),
    routes: [
      {
        path: '/',
        component: ComponentCreator('/', '6e1'),
        routes: [
          {
            path: '/',
            component: ComponentCreator('/', '0d1'),
            routes: [
              {
                path: '/alpha-testnet',
                component: ComponentCreator('/alpha-testnet', 'bff'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/community',
                component: ComponentCreator('/community', 'cac'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/core-concepts/composable-modular-networks',
                component: ComponentCreator('/core-concepts/composable-modular-networks', 'd37'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/core-concepts/cryptography/2pc-mpc',
                component: ComponentCreator('/core-concepts/cryptography/2pc-mpc', '3c7'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/core-concepts/cryptography/mpc',
                component: ComponentCreator('/core-concepts/cryptography/mpc', '2c7'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/core-concepts/dwallets',
                component: ComponentCreator('/core-concepts/dwallets', 'd0c'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/core-concepts/multi-chain-vs-cross-chain',
                component: ComponentCreator('/core-concepts/multi-chain-vs-cross-chain', '94e'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/core-concepts/noncollusive-and-decentralized',
                component: ComponentCreator('/core-concepts/noncollusive-and-decentralized', '23c'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/core-concepts/state-proofs',
                component: ComponentCreator('/core-concepts/state-proofs', 'cf6'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/connect',
                component: ComponentCreator('/developers-guide/getting-started/connect', '316'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/dwallet-network-environment',
                component: ComponentCreator('/developers-guide/getting-started/dwallet-network-environment', '430'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/encryption-key',
                component: ComponentCreator('/developers-guide/getting-started/encryption-key', 'a3b'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/examples/bitcoin-multisig-solidity',
                component: ComponentCreator('/developers-guide/getting-started/examples/bitcoin-multisig-solidity', '9c9'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/examples/bitcoin-multisig-sui-move',
                component: ComponentCreator('/developers-guide/getting-started/examples/bitcoin-multisig-sui-move', '8a7'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/examples/broadcast-bitcoin-tx',
                component: ComponentCreator('/developers-guide/getting-started/examples/broadcast-bitcoin-tx', 'f19'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/examples/broadcast-ether-tx',
                component: ComponentCreator('/developers-guide/getting-started/examples/broadcast-ether-tx', 'feb'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/examples/multi-chain-atomic-swap',
                component: ComponentCreator('/developers-guide/getting-started/examples/multi-chain-atomic-swap', '283'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/examples/multi-chain-lending',
                component: ComponentCreator('/developers-guide/getting-started/examples/multi-chain-lending', '7ae'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/future-transaction',
                component: ComponentCreator('/developers-guide/getting-started/future-transaction', 'e4b'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/get-address',
                component: ComponentCreator('/developers-guide/getting-started/get-address', 'b64'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/get-tokens',
                component: ComponentCreator('/developers-guide/getting-started/get-tokens', 'de7'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/install-dwallet',
                component: ComponentCreator('/developers-guide/getting-started/install-dwallet', 'dd5'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/local-network',
                component: ComponentCreator('/developers-guide/getting-started/local-network', 'dee'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/user-share-encryption',
                component: ComponentCreator('/developers-guide/getting-started/user-share-encryption', '11f'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/getting-started/your-first-dwallet',
                component: ComponentCreator('/developers-guide/getting-started/your-first-dwallet', 'f3c'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/lightclients/ether-lightclient',
                component: ComponentCreator('/developers-guide/lightclients/ether-lightclient', '158'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/developers-guide/lightclients/sui-lightclient',
                component: ComponentCreator('/developers-guide/lightclients/sui-lightclient', '96e'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/operators-guide',
                component: ComponentCreator('/operators-guide', '730'),
                exact: true,
                sidebar: "dwalletSidebar"
              },
              {
                path: '/',
                component: ComponentCreator('/', '93f'),
                exact: true,
                sidebar: "dwalletSidebar"
              }
            ]
          }
        ]
      }
    ]
  },
  {
    path: '*',
    component: ComponentCreator('*'),
  },
];
