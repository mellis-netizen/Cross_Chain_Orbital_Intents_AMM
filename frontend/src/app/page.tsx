import { redirect } from 'next/navigation'

export default function HomePage() {
  // Redirect to orbital AMM as the main interface
  redirect('/orbital')
}