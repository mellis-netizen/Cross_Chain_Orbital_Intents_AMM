import { redirect } from 'next/navigation'

export default function HomePage() {
  // Redirect to swap page as the main interface
  redirect('/swap')
}