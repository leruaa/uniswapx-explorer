import Header from '../components/header';
import Providers from './providers'
import './globals.css';

export default function Layout({
  children,
}: {
  children: React.ReactNode
}) {
  return (
    <html lang="en">
      <body>
        <Providers>
          <main>
            <Header />
            <section className="container mx-auto">
              {children}
            </section>
          </main>
        </Providers>
      </body>
    </html>
  )
}