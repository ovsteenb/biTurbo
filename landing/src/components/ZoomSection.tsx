"use client";

import { motion, useScroll, useTransform, type MotionValue } from "framer-motion";
import { useRef, useEffect, useState, type ReactNode } from "react";
import { cn } from "@/lib/cn";

type ZoomSectionProps = {
  id: string;
  index: string;
  eyebrow: string;
  title: ReactNode;
  description: ReactNode;
  visual: ReactNode;
  variant?: "moss" | "amber" | "sky" | "lilac";
  align?: "left" | "right" | "center";
};

const variantText: Record<NonNullable<ZoomSectionProps["variant"]>, string> = {
  moss: "text-moss",
  amber: "text-amber",
  sky: "text-sky",
  lilac: "text-lilac",
};

const variantBorder: Record<NonNullable<ZoomSectionProps["variant"]>, string> = {
  moss: "border-moss/30",
  amber: "border-amber/30",
  sky: "border-sky/30",
  lilac: "border-lilac/30",
};

export function ZoomSection({
  id,
  index,
  eyebrow,
  title,
  description,
  visual,
  variant = "moss",
  align = "left",
}: ZoomSectionProps) {
  const ref = useRef<HTMLDivElement>(null);
  const [mounted, setMounted] = useState(false);

  useEffect(() => {
    setMounted(true);
  }, []);

  const { scrollYProgress } = useScroll({
    target: ref,
    offset: ["start end", "end start"],
  });

  const visualScale = useTransform(scrollYProgress, [0, 0.4, 0.6, 1], [0.7, 1.0, 1.1, 0.92]);
  const visualOpacity = useTransform(scrollYProgress, [0, 0.1, 0.9, 1], [0, 1, 1, 0.5]);
  const visualY = useTransform(scrollYProgress, [0, 0.5, 1], [40, 0, -20]);

  const headlineY = useTransform(scrollYProgress, [0, 0.5, 1], [50, 0, -30]);
  const headlineScale = useTransform(scrollYProgress, [0, 0.5, 1], [0.94, 1, 1.02]);
  const headlineOpacity = useTransform(scrollYProgress, [0, 0.08, 0.9, 1], [0, 1, 1, 0.4]);

  const descOpacity = useTransform(scrollYProgress, [0.15, 0.3, 0.9, 1], [0, 1, 1, 0.3]);
  const descY = useTransform(scrollYProgress, [0.15, 0.3, 0.9, 1], [20, 0, 0, -10]);

  return (
    <section
      ref={ref}
      id={id}
      className="relative min-h-[140vh] overflow-hidden"
    >
      <div
        className={cn(
          "pointer-events-none absolute inset-0 bg-gradient-radial",
          variant === "moss" && "gradient-radial-moss",
          variant === "amber" && "gradient-radial-amber",
          variant === "sky" && "gradient-radial-sky",
          variant === "lilac" && "gradient-radial-lilac"
        )}
      />
      <div className="grid-lines pointer-events-none absolute inset-0 opacity-30" />

      <div className="sticky top-0 flex h-screen items-center overflow-hidden">
        <div className="relative z-10 mx-auto grid w-full max-w-7xl grid-cols-1 gap-12 px-6 lg:grid-cols-12 lg:items-center">
          <MotionBlock
            mounted={mounted}
            className={cn("lg:col-span-6", align === "right" && "lg:order-2", align === "center" && "lg:col-span-12 lg:text-center")}
            y={headlineY}
            scale={headlineScale}
            opacity={headlineOpacity}
          >
            <div className="mb-4 flex items-center gap-3">
              <span className={cn("font-mono text-xs uppercase tracking-[0.2em]", variantText[variant])}>
                {index} · {eyebrow}
              </span>
              <div className={cn("h-px flex-1 bg-gradient-to-r from-current to-transparent opacity-30", variantText[variant])} />
            </div>
            <h2 className="font-display text-[clamp(2.5rem,6.5vw,5.5rem)] font-extrabold leading-[0.95] tracking-[-0.03em] text-ink text-balance">
              {title}
            </h2>
            <MotionBlock
              mounted={mounted}
              className="mt-8 max-w-xl text-pretty text-lg text-ink-200/80 md:text-xl"
              opacity={descOpacity}
              y={descY}
            >
              {description}
            </MotionBlock>
          </MotionBlock>

          <MotionBlock
            mounted={mounted}
            className={cn("lg:col-span-6", align === "right" && "lg:order-1", align === "center" && "lg:col-span-12")}
            scale={visualScale}
            opacity={visualOpacity}
            y={visualY}
          >
            <div className={cn("relative aspect-[4/3] w-full overflow-hidden rounded-2xl border bg-ink-800/40 backdrop-blur-sm", variantBorder[variant])}>
              {visual}
            </div>
          </MotionBlock>
        </div>
      </div>
    </section>
  );
}

function MotionBlock({
  mounted,
  className,
  y,
  scale,
  opacity,
  children,
}: {
  mounted: boolean;
  className?: string;
  y?: MotionValue<number>;
  scale?: MotionValue<number>;
  opacity?: MotionValue<number>;
  children: ReactNode;
}) {
  if (!mounted) {
    return <div className={className}>{children}</div>;
  }
  return (
    <motion.div className={className} style={{ y, scale, opacity }}>
      {children}
    </motion.div>
  );
}
