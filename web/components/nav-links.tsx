"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

type NavLink = { href: string; label: string; cn?: string };

export function NavLinks({ links, isZh }: { links: NavLink[]; isZh: boolean }) {
  const pathname = usePathname();

  return (
    <nav className="hidden md:flex items-center gap-7">
      {links.map((l) => {
        const isActive = pathname === l.href || pathname.startsWith(`${l.href}/`);
        return (
          <Link key={l.href} href={l.href} className="nav-link group" aria-current={isActive ? "page" : undefined}>
            <span>{l.label}</span>
            {!isZh && l.cn && (
              <span className="font-cjk text-[0.66rem] ml-1.5 text-ink-mute">{l.cn}</span>
            )}
          </Link>
        );
      })}
    </nav>
  );
}
