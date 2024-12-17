import Image from 'next/image';
import Spike from "@/public/assets/spike.png";
const ErrorComponent = ({ error }) => {
  return (
    <div className="flex justify-center items-center w-full h-full ">
      <div>
        <Image
          src={Spike}
          width={500}
          height={500}
          alt="Error"

        />
        <p className='w-full text-center text-lg text-red-500'>{error}</p>
      </div>

    </div>
  );
}

export default ErrorComponent;
