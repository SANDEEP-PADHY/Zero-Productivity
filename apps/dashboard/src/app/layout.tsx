import type { Metadata } from "next";
import "./globals.css";
import { Navigation } from "@/components/navigation";

export const metadata: Metadata = {
  title: "Zero Productivity Dashboard",
  description: "Track and analyze your productivity across devices.",
};

export default function RootLayout({
  children,
}: Readonly<{
  children: React.ReactNode;
}>) {
  return (
    <html lang="en">
      <body className="antialiased bg-[var(--color-zp-white)] text-[var(--color-zp-black)] min-h-screen flex">
        <Navigation />
        <main className="flex-1 p-8 ml-64 overflow-y-auto">
          {children}
        </main>
      </body>
    </html>
  );
}
