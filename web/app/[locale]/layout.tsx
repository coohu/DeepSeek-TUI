import type { Metadata } from "next";
import { Nav } from "@/components/nav";
import { Footer } from "@/components/footer";
import { locales, type Locale } from "@/lib/i18n/config";

export function generateStaticParams() {
  return locales.map((locale) => ({ locale }));
}

export async function generateMetadata({ params }: { params: Promise<{ locale: string }> }): Promise<Metadata> {
  const { locale } = await params;
  const isZh = locale === "zh";
  return {
    title: isZh ? "CodeWhale · DeepSeek V4 智能体运行框架" : "CodeWhale · DeepSeek V4 智能体运行框架",
    description: isZh
      ? "基于 DeepSeek V4 的开源终端编程智能体。支持 100 万 token 上下文、MCP 协议、沙箱执行。"
      : "Terminal-native coding agent built on DeepSeek V4. Open source. Community site for installation, docs, roadmap, and live activity from the coohu/ repo.",
    metadataBase: new URL("https://.com"),
    openGraph: {
      title: "CodeWhale",
      description: isZh
        ? "DeepSeek V4 的最强智能体运行框架。宪政层级、结构化信任、验证与恢复。"
        : "The most agentic harness for DeepSeek V4. Constitutional hierarchy, structured trust, verification, and recovery.",
      url: "https://deepseek.net",
      siteName: "CodeWhale",
      type: "website",
    },
    twitter: { card: "summary_large_image" },
    alternates: {
      languages: {
        en: "/en",
        zh: "/zh",
      },
    },
  };
}

export default async function LocaleLayout({
  children,
  params,
}: {
  children: React.ReactNode;
  params: Promise<{ locale: string }>;
}) {
  const { locale } = await params;

  return (
    <>
      <Nav locale={locale as Locale} />
      <main>{children}</main>
      <Footer locale={locale as Locale} />
    </>
  );
}
