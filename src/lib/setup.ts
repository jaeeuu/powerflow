import type { App } from 'vue'
import messages from '@intlify/unplugin-vue-i18n/messages'
import { MotionPlugin } from '@vueuse/motion'
import { createPlugin as createTauriPiniaPlugin } from '@tauri-store/pinia'
import { createI18n } from 'vue-i18n'
import '../assets/index.css'

const i18n = createI18n({
  locale: 'en',
  fallbackLocale: 'en',
  messages,
})

export function setup(entry: Component, fn?: (app: App<Element>) => void) {
  const app = createApp(entry)
    .use(MotionPlugin)
    .use(
      createPinia()
        .use(createTauriPiniaPlugin()),
    )
    .use(i18n)

  if (fn) {
    fn(app)
  }

  app.mount('#app')
}
