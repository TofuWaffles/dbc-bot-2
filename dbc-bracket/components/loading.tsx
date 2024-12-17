import Image from 'next/image';
import logo from "@/public/assets/logo.gif";
const Loading: React.FC = () => {
    return (
        <div className="w-full h-full justify-center items-center flex">
            <Image
                src={logo}
                width={200}
                height={200}
                alt="Loading animation"
            />
        </div>
    )
}

export default Loading;