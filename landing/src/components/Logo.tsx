import Image from "next/image";
import { cn } from "@/lib/cn";

type LogoProps = {
  /** Height in pixels. Width is derived from the source aspect ratio. */
  size?: number;
  /** Show just the rounded-square mark, no wordmark. */
  markOnly?: boolean;
  className?: string;
  /** Override the default source. */
  src?: string;
  priority?: boolean;
};

const ASPECT_FULL = 600 / 218;
const ASPECT_MARK = 1;

export function Logo({
  size = 32,
  markOnly = false,
  className,
  src,
  priority = false,
}: LogoProps) {
  const aspect = markOnly ? ASPECT_MARK : ASPECT_FULL;
  const w = markOnly ? size : Math.round(size * aspect);
  const h = size;
  const imgSrc = src ?? (markOnly ? "/logo-mark.png" : "/logo-full-600.png");

  return (
    <span className={cn("inline-flex items-center", className)}>
      <Image
        src={imgSrc}
        alt="biTurbo"
        width={w * 2}
        height={h * 2}
        sizes={`${w}px`}
        quality={100}
        priority={priority}
        className="block"
        style={{ width: w, height: h }}
      />
    </span>
  );
}
