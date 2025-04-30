
import { createContext, PropsWithChildren, useContext, useState } from "react"

type Props = Record<string, unknown>;

type ContextProps = {
  region: string | null;
  setRegion: (region: string) => void;
}

const defaultValues = {
  region: null,
  setRegion: () => undefined,
}

const RegionContext = createContext<ContextProps>(defaultValues);

export const RegionProvider = ({ children }: PropsWithChildren<Props>) => {
  const [region, setRegion] = useState<string | null>(null);

  return (
    <RegionContext.Provider
      value={{
        region,
        setRegion,
      }}
    >
      {children}
    </RegionContext.Provider>
  )
}

export const useRegion = () => {
  return useContext(RegionContext)
}